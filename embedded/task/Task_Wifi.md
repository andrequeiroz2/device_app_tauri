# Task: WiFi (wifi.json)

## Status

| Fase | Método | Status |
|------|--------|--------|
| 1 | Constantes e estrutura base | ✅ Done |
| 2 | `write_init_config()` | ✅ Done |
| 3 | `add_wifi()` | ✅ Done |
| 4 | `set_default()` | ✅ Done — movida para `tool/wifi.py` |
| 5 | `connect()` | ✅ Done |
| 6 | `connect_from_config()` | ✅ Done |
| 7 | `get_status()` | ✅ Done |

---

## Objetivo

Implementar módulo de gerenciamento de redes WiFi conhecidas para o device embarcado, criando e gerenciando `/wifi.json`. O device deve tentar reconectar automaticamente ao reboot usando a lista de redes registradas, priorizando sempre a rede marcada como `default`.

---

## Análise do Código de Referência

**Código fornecido:** classe `WanConfig` em `wan_config.json` (exemplo de implementação anterior).

### Diferenças para nosso projeto

| Aspecto | WanConfig (referência) | Nosso `config_wifi` |
|---------|------------------------|---------------------|
| Organização | Classe estática | Funções de módulo (padrão do projeto) |
| Arquivo de config | `/wan_config.json` | `/wifi.json` |
| Logging | `print()` direto | `log`/`log_err` de `tool/log` |
| I/O de arquivo | `open()` inline | `read_json`/`write_json` de `tool/read` e `tool/write` |
| File exists | `os.stat()` inline | `file_exists` de `tool/load` |
| `encoding` | `encoding="utf-8"` (incompatível MicroPython) | Sem `encoding` |
| Type hints | Python 3.10+ (`str \| None`) | Sem type hints (`\|` não suportado) |
| Async | Todas as funções `async` | Sync onde possível; async apenas em operações de rede |

### Bug no código de referência (`connect_to_wifi`)

O `for` loop verifica `wlan.isconnected()`, atualiza o default e **não retorna `True`** — cai no fim do loop e retorna `False` mesmo em caso de sucesso:

```python
# BUG: falta return True após conexão confirmada
for _ in range(timeout * 2):
    if wlan.isconnected():
        WanConfig._last_password = password
        wan_update_default = await WanConfig.update_default_network(ssid)
        # ← sem return True aqui
    await asyncio.sleep(0.5)
return False  # sempre retorna False
```

**Correção aplicada:** `return True` imediatamente após `set_default(ssid)` dentro do loop.

---

## Estrutura do arquivo `/wifi.json`

```json
{
  "wifi": [
    {"ssid": "MinhaRede", "pass": "senha123", "default": true},
    {"ssid": "RedeBackup", "pass": "",         "default": false}
  ]
}
```

**Regras:**
- Pode haver `N` entradas na lista
- **Apenas uma** entrada pode ter `default: true` em qualquer momento
- `pass` pode ser string vazia (rede aberta)
- Entradas com `ssid` vazio são ignoradas na tentativa de conexão
- Toda vez que uma conexão é estabelecida com sucesso, esse ssid passa a ser `default: true`

---

## API Planejada — `embedded/esp/wifi/config_wifi.py`

### Constantes

```python
WIFI_FILE   = "/wifi.json"          # path do arquivo de configuração
_INIT_ENTRY = {"ssid": "", "pass": "", "default": False}  # entrada vazia inicial
```

### Helpers privados — `wifi/config_wifi.py`

| Função | Descrição |
|--------|-----------|
| `_is_entry_valid(entry)` | Retorna `True` se `ssid` é string não vazia |
| `_remove_wifi(ssid)` | Remove entrada por ssid — usado para rollback quando adoção falha após `add_wifi` |

### Funções públicas — `wifi/config_wifi.py`

| Função | Tipo | Retorno | Descrição |
|--------|------|---------|-----------|
| `write_init_config()` | sync | `bool` | Cria `/wifi.json` com `_INIT_ENTRY` se não existir ou estiver corrompido |
| `add_wifi(ssid, password)` | sync | `bool` | Upsert: insere nova entrada ou atualiza `pass` se ssid já existir. Não altera `default` |
| `connect(ssid, password, timeout)` | async | `bool` | Tenta conectar; em sucesso chama `set_default`; falha de `set_default` loga aviso mas não cancela conexão |
| `connect_from_config()` | async | `bool` | Iteração única: encontra `default=True` e coleta demais; tenta default primeiro, depois os demais em ordem |
| `get_status()` | sync | `dict` | Retorna `{connected, ssid, ip, netmask, gateway, dns, rssi}`; `rssi` com try/except (compatibilidade de board) |

### Funções públicas — `tool/wifi.py`

| Função | Assinatura | Retorno | Descrição |
|--------|------------|---------|-----------|
| `set_default` | `(wifi_file, ssid)` | `bool` | Define `default=True` para o ssid dado, `False` para todos os outros. Recebe o path como argumento (padrão do projeto) |

### `get_status()` — estrutura de retorno

```python
{
    "connected": False,
    "ssid":      None,
    "ip":        None,
    "netmask":   None,
    "gateway":   None,
    "dns":       None,
    "rssi":      None
}
```

---

## Fases de Implementação

### Fase 1 ❌ — Constantes e estrutura base

```python
WIFI_FILE = "/wifi.json"
_MODULE   = "config_wifi"
```

Imports necessários:
```python
import network
import uasyncio as asyncio
from tool.load import file_exists
from tool.log  import log, log_err
from tool.read import read_json
from tool.write import write_json
```

---

### Fase 2 ❌ — `write_init_config()`

**Lógica:**

```
1. file_exists(WIFI_FILE) == False?
   → gravar {"wifi": [{"ssid": "", "pass": "", "default": false}]}
   → return True/False conforme write_json

2. read_json(WIFI_FILE) == None? (arquivo existe mas corrompido)
   → mesmo fluxo acima

3. "wifi" in config e isinstance(config["wifi"], list)?
   → log "already initialized"
   → return True

4. Estrutura inválida:
   → recriar com entrada vazia
   → return True/False
```

---

### Fase 3 ❌ — `add_wifi(ssid, password)`

**Lógica:**

```
1. Validar: ssid deve ser str não vazio → log_err e return False
2. Validar: password deve ser str → log_err e return False
3. read_json(WIFI_FILE) → config
4. config None ou sem chave "wifi"? → inicializar {"wifi": []}
5. Buscar ssid na lista:
   - Encontrou → atualizar "pass" (não alterar "default")
   - Não encontrou → append {"ssid": ssid, "pass": password, "default": false}
6. write_json(WIFI_FILE, config) → return bool
```

> **`default` não é alterado aqui.** Ele é gerenciado exclusivamente por `set_default`, que é chamado por `connect` somente após conexão bem sucedida.

---

### Fase 4 ✅ — `set_default(wifi_file, ssid)` — em `tool/wifi.py`

**Decisão:** movida para `tool/wifi.py` seguindo o padrão do projeto — funções genéricas de manipulação de arquivo recebem o path como argumento.

**Lógica:**

```
1. read_json(wifi_file) → config
2. config None ou sem "wifi"? → log_err e return False
3. Iterar lista:
   - entry["ssid"] == ssid → entry["default"] = True; updated = True
   - caso contrário         → entry["default"] = False
4. updated == False? → log_err "ssid not found" e return False
5. write_json(wifi_file, config) → return bool
```

---

### Fase 5 ✅ — `connect(ssid, password, timeout=10)`

**Lógica:**

```
1. wlan = network.WLAN(network.STA_IF); wlan.active(True)
2. Se já conectado no mesmo ssid:
   → log "already connected"
   → set_default(WIFI_FILE, ssid) — falha loga aviso, não cancela
   → return True
3. Se conectado em outro ssid → desconectar; sleep(2)
4. wlan.connect(ssid, password)
5. Loop até timeout * 2 iterações (sleep 0.5s cada):
   - wlan.isconnected()?
     → log "connected ip=..."
     → set_default(WIFI_FILE, ssid) — falha loga aviso, não cancela
     → return True
6. wlan.disconnect()
7. log_err "connection timeout"
8. return False
```

> **Bug fix do exemplo:** `return True` ocorre **dentro** do loop assim que `wlan.isconnected()` for `True`.  
> **`set_default` não é fatal:** falha é logada como `log_err` mas a função retorna `True` pois o WiFi está conectado.

---

### Fase 6 ✅ — `connect_from_config()`

**Entry point para `main.py`.**

**Lógica:**

```
1. read_json(WIFI_FILE) → config
2. config None ou wifi_list vazia? → log_err e return False
3. Iteração única: percorre wifi_list uma vez
   - entry["default"] is True  → default_net = entry
   - caso contrário             → other_nets.append(entry)
4. networks_to_try = [default_net] + other_nets  (ou só other_nets se não há default)
5. Para cada entry em networks_to_try:
   - _is_entry_valid(entry) == False? → skip (ssid vazio)
   - await connect(ssid, password)?
     → log "connected via ssid=..."
     → return True
6. log_err "could not connect to any network"
7. return False
```

> **Otimização:** iteração única substitui duas list comprehensions — metade das iterações e menos alocações de lista (relevante em MicroPython).

---

### Fase 7 ✅ — `get_status()`

**Lógica:**

```
1. Inicializar _obj com todos os campos None / connected=False
2. wlan = network.WLAN(network.STA_IF)
3. wlan.active(True)
4. wlan.isconnected() == False?
   → log_err "not connected"
   → return _obj
5. Preencher _obj: ssid, ip, netmask, gateway, dns, rssi
6. log "connected ip=..."
7. return _obj
```

---

## Integração com `main.py`

```python
from config.config    import write_init_config as config_init
from wifi.config_wifi import write_init_config as wifi_init
from wifi.config_wifi import connect_from_config

async def main():
    config_init()
    wifi_init()
    await connect_from_config()
    # ...
```

---

## Integração com o fluxo de adoção (`set_config`)

`wifi_ssid` e `wifi_password` **não são gravados em `config.json`**. Todo dado WiFi reside exclusivamente em `/wifi.json`.

**Regra principal:** o WiFi deve conectar com sucesso **antes** de qualquer gravação. Se a conexão falhar, nenhum dado é persistido.

### Fluxo do handler do comando `set_config`

```
1. validate_adoption_data(data)
   → False? rejeitar (nenhuma gravação)

2. add_wifi(data["wifi_ssid"], data.get("wifi_password", ""))
   → False? rejeitar
   → Adiciona entrada em wifi.json (sem alterar default ainda)

3. await connect(data["wifi_ssid"], data.get("wifi_password", ""))
   → False? rollback: remover entrada de wifi.json → rejeitar
   → Conectou: set_default(ssid) é chamado internamente por connect()

4. adopt_device(data)
   → False? rollback: remover entrada de wifi.json → rejeitar

5. Responder sucesso
```

> A ordem `add_wifi → connect` é necessária porque `connect()` chama `set_default()` internamente, que precisa encontrar o ssid já registrado em `wifi.json`.

### Separação de responsabilidades no payload de `set_config`

| Campo | Destino | Módulo responsável |
|-------|---------|-------------------|
| `user_uuid` | `config.json` | `config/config.py` |
| `device_uuid` | `config.json` (write-once) | `config/config.py` |
| `device_name` | `config.json` | `config/config.py` |
| `topic` | `config.json` | `config/config.py` |
| `broker_url` | `config.json` | `config/config.py` |
| `adopted_status` | `config.json` | `config/config.py` |
| `adopted_status_desc` | `config.json` | `config/config.py` |
| `wifi_ssid` | `wifi.json` | `wifi/config_wifi.py` |
| `wifi_password` | `wifi.json` | `wifi/config_wifi.py` |

> `wifi_ssid` está em `REQUIRED_ADOPTION_KEYS` — obrigatório no payload, mas roteado para `wifi.json` pelo handler, não por `adopt_device`.

---

## Estrutura final esperada

```
embedded/esp/
├── config/
│   └── config.py          ✅ config.json genérico
├── mqtt/
│   └── mqtt_config.py     ✅ broker URL, parse, params
├── wifi/
│   └── config_wifi.py     ✅ gerenciamento de WiFi
└── tool/
    ├── load.py             ✅ file_exists
    ├── log.py              ✅ log, log_err
    ├── network.py          ✅ get_mac_address
    ├── read.py             ✅ read_json
    ├── validate.py         ✅ validate_adoption_data
    ├── wifi.py             ✅ set_default
    └── write.py            ✅ write_json
```

**Separação de responsabilidades:**

| Arquivo | Responsabilidade | Acessa WiFi HW? |
|---------|------------------|-----------------|
| `config/config.py` | Ler/gravar `config.json`, adoção | ❌ Não |
| `mqtt/mqtt_config.py` | Broker URL, params de conexão | ❌ Não |
| `wifi/config_wifi.py` | Lista de redes, conexão, status, rollback | ✅ Sim |
| `tool/wifi.py` | `set_default` — manipulação genérica de `wifi.json` | ❌ Não |

---

## Decisões

| Questão | Decisão |
|---------|---------|
| **Async vs Sync** | Sync para I/O de arquivo; async apenas para `connect` e `connect_from_config` (usam `asyncio.sleep`) |
| **Apenas um default** | Garantido por `set_default`: itera toda a lista antes de gravar |
| **Rede aberta** | `pass` pode ser string vazia — validação aceita `str`, não `str não vazio` |
| **Entradas inválidas** | Entradas com `ssid` vazio são ignoradas (não removidas) |
| **Path do arquivo** | `/wifi.json` — path absoluto na raiz do filesystem |
| **Bug do exemplo** | `return True` dentro do loop de polling, não fora |
