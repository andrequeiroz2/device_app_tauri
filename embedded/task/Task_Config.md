# Task: Config (config.json)

## Status

| Fase | Método | Status |
|------|--------|--------|
| 1 | Constantes e estrutura base | ✅ Done |
| 2 | `load_config()` | ✅ Done |
| 3 | `read_config()` | ✅ Done |
| 4 | `get_mac_address()` | ✅ Done |
| 5 | `write_init_config()` | ✅ Done |
| 6 | `adopt_device()` | ✅ Done |
| 7 | `update_broker_url()` | ✅ Done — em `mqtt/mqtt_config.py` |

---

## Objetivo

Implementar módulo de configuração persistente para o device embarcado, criando e gerenciando `config.json` conforme `embedded/ROLE.md`.

---

## Análise do Código de Referência

**Projeto analisado:** `/home/andre/RustroverProjects/device_app_micropython`

### Estrutura encontrada

| Arquivo | Responsabilidade |
|---------|------------------|
| `config_tool.py` | Classe `DeviceConfig` com métodos para config |
| `device_config.json` | Arquivo de persistência |

### Diferenças para nosso projeto

| Aspecto | device_app_micropython | Nosso embedded |
|---------|------------------------|----------------|
| Arquivo | `device_config.json` | `config.json` |
| WiFi | Separado em `wan_config.json` | Separado em `wifi.json` — gerenciado por `wifi/config_wifi.py` |
| Adoção | `write_adopted_device(...)` 5 params | `update_config(data: dict)` recebe objeto completo |
| device_scale | Typo `"humidty"` | `"humidity"` (correto) |
| Async | Todos métodos `async` | Async ✅ |

---

## API Implementada — `embedded/esp/config/config.py`

### Constantes públicas

```python
CONFIG_FILE = "/config.json"

BOARDER_TYPE = "ESP32"
DEVICE_TYPE  = "Sensor"
SENSOR_TYPE  = "DHT11"
ACTUATOR_TYPE = ""
DEVICE_SCALE  = [["temperature", "C"], ["humidity", "%"]]
ADOPTED_STATUS = 0
ADOPTED_STATUS_DESC = "not_adopted"
```

### Classe `AdoptedStatus`

```python
class AdoptedStatus:
    UNADOPTED = 0
    ADOPTED   = 1
    DESC = {0: "not_adopted", 1: "adopted"}
```

### Helpers privados (adicionados na implementação)

| Função | Assinatura | Descrição |
|--------|------------|-----------|
| `_is_config_complete` | `(config: dict) -> bool` | Verifica se todos os campos device-side obrigatórios existem e são válidos |
| `_build_init_config` | `(mac: str) -> dict` | Monta dict inicial com defaults + MAC. Campos de adoção inicializados como `""` |
| `_validate_update_data` | `(data: dict) -> bool` | Valida payload de `set_config`: campos obrigatórios não vazios, `adopted_status` em `{0,1}` |

**Constantes de validação:**

```python
REQUIRED_ADOPTION_KEYS = ("user_uuid", "device_uuid", "topic", "broker_url", "wifi_ssid")
ADOPTION_KEYS = ("device_name", "user_uuid", "topic", "broker_url", "adopted_status", "adopted_status_desc")
```

> `wifi_ssid` está em `REQUIRED_ADOPTION_KEYS` (validação obrigatória) mas **não** em `ADOPTION_KEYS` — é roteado para `wifi.json` pelo handler, não gravado em `config.json`.  
> `wifi_password` **não** aparece em nenhuma das constantes de config — reside exclusivamente em `wifi.json`.  
> `device_uuid` **não** está em `ADOPTION_KEYS` — tratado separadamente como write-once.

### Funções públicas

| Função | Assinatura | Retorno | Descrição |
|--------|------------|---------|-----------|
| `load_config` | `async () -> bool` | `True` se arquivo existe | Usa `os.stat()`, retorna `False` em `OSError` |
| `read_config` | `async () -> dict \| None` | config dict ou `None` | Abre e parseia `config.json`; retorna `None` em erro |
| `get_mac_address` | `async () -> str \| None` | `"3C:71:BF:4D:DB:0C"` ou `None` | Obtém MAC via `network.WLAN(STA_IF)`, formata uppercase |
| `write_init_config` | `async () -> bool` | `True` em sucesso | Garante que `config.json` existe e está completo |
| `adopt_device` | `async (data: dict) -> bool` | `True` em sucesso | Adota o device aplicando o payload de `set_config` |

---

## Fases de Implementação

### Fase 1 ✅ — Constantes e estrutura base

Todas as constantes e a classe `AdoptedStatus` definidas conforme especificado.

---

### Fase 2 ✅ — `load_config()`

```python
async def load_config() -> bool:
    try:
        os.stat(CONFIG_FILE)
        return True
    except OSError as e:
        print("config: load_config error:", e)
        return False
```

**Conforme especificado.** Usa `os.stat`, loga erro, retorna `False` em falha.

---

### Fase 3 ✅ — `read_config()`

```python
async def read_config() -> dict | None:
    try:
        with open(CONFIG_FILE, "r", encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError):
        return None
```

**Conforme especificado.** Usa `with`, captura `OSError` e `ValueError`, retorna `None`.

---

### Fase 4 ✅ — `get_mac_address()`

```python
async def get_mac_address() -> str | None:
    try:
        wlan = network.WLAN(network.STA_IF)
        if not wlan.active():
            wlan.active(True)
        mac = wlan.config("mac")
        return ":".join("{:02X}".format(b) for b in mac)
    except (OSError, AttributeError):
        return None
```

**Conforme especificado.** Ativa interface se necessário, formata uppercase.

---

### Fase 5 ✅ — `write_init_config()`

**Lógica implementada** (difere da referência do doc anterior que tinha contradição):

```
1. load_config() == False?
   → get_mac_address()
   → _build_init_config(mac)
   → gravar config.json
   → return True

2. read_config() == None? (arquivo existe mas inválido)
   → mesmo fluxo acima

3. _is_config_complete(config) == True?
   → return True (já completo, nada a fazer)

4. Config incompleta:
   → get_mac_address()
   → _build_init_config(mac)
   → gravar config.json
   → return True
```

> **Correção ao doc original:** O passo 2 da "Análise do Código de Referência" dizia "Se não existe → erro e retorna False", o que contradiz o objetivo da função. A implementação correta **cria o arquivo** quando não existe.

**Campos verificados por `_is_config_complete`:**
`boarder_type`, `mac_address`, `device_type`, `sensor_type`, `actuator_type`, `adopted_status`, `adopted_status_desc`, `device_scale`

Validações extras além da presença da chave:
- `mac_address` não pode ser string vazia
- `device_scale` deve ser `list`

---

### Fase 6 ✅ — `adopt_device()`

**Decisão de nomenclatura:** a função foi renomeada de `update_config` para `adopt_device` pois o processo é de **adoção**, não de atualização genérica.

**Responsabilidade de `adopt_device`:** gravar apenas os campos de `config.json`. A conexão WiFi e a gravação em `wifi.json` são responsabilidade do handler do `set_config` — `adopt_device` é chamado **depois** que o WiFi já conectou.

**Lógica implementada:**

```
1. validate_adoption_data(data) == False? → return False
   (valida: user_uuid, device_uuid, topic, broker_url, wifi_ssid não vazios)

2. read_config() == None? → return False

3. Guarda de re-adoção:
   - SE adopted_status == ADOPTED
   - E data["user_uuid"] != config["user_uuid"]
   → return False  ← bloqueia apenas usuário DIFERENTE (idempotente para mesmo usuário)

4. Gravar campos via ADOPTION_KEYS

5. device_uuid: set only if config["device_uuid"] == ""
   (write-once — nunca sobrescrever)

6. Gravar config.json → return True
```

**Fluxo completo de adoção (handler do `set_config` em `main.py`):**

```
1. validate_adoption_data(data)        → False? rejeitar
2. add_wifi(wifi_ssid, wifi_password)  → False? rejeitar
3. await connect(wifi_ssid, password)  → False? rollback wifi.json → rejeitar
4. adopt_device(data)                  → False? rollback wifi.json → rejeitar
5. Responder sucesso
```

> WiFi conecta **antes** de qualquer gravação. Se a conexão falhar, nada é persistido.

**Constantes renomeadas:**

| Antes | Depois |
|-------|--------|
| `_REQUIRED_UPDATE_KEYS` | `_REQUIRED_ADOPTION_KEYS` |
| `_UPDATE_KEYS` | `_ADOPTION_KEYS` |
| `_validate_update_data` | `_validate_adoption_data` |

**`REQUIRED_ADOPTION_KEYS`** (campos obrigatórios no payload — validação):
`user_uuid`, `device_uuid`, `topic`, `broker_url`, `wifi_ssid`

**`ADOPTION_KEYS`** (campos gravados em `config.json`):
`device_name`, `user_uuid`, `topic`, `broker_url`, `adopted_status`, `adopted_status_desc`

> `device_name` removido dos obrigatórios — cosmético, não impede o device de funcionar.  
> `device_uuid` é **imutável** após a primeira adoção (write-once).  
> `wifi_ssid` e `wifi_password` são validados mas **não gravados** em `config.json` — roteados para `wifi.json` pelo handler do comando `set_config`.

---

### Fase 7 ✅ — `update_broker_url()` — DONE

**Decisão:** toda lógica de broker reside em `mqtt/mqtt_config.py`. O `config.py` **não** contém nada relativo a broker — apenas lê/grava o `config.json` de forma genérica.

**Arquivo:** `embedded/esp/mqtt/mqtt_config.py`

#### API implementada

| Função | Assinatura | Retorno | Descrição |
|--------|------------|---------|-----------|
| `_parse_broker_url` | `(broker_url: str) -> tuple \| None` | `(host, port, ssl)` ou `None` | Parseia `mqtt://` ou `mqtts://`. Privado. |
| `update_broker_url` | `async (broker_url: str) -> bool` | `True` em sucesso | Valida URL, atualiza `broker_url` no `config.json` |
| `get_broker_params` | `async () -> tuple \| None` | `(host, port, ssl)` ou `None` | Lê config e retorna parâmetros de conexão |

#### `_parse_broker_url` — formatos suportados

| Input | host | port | ssl |
|-------|------|------|-----|
| `mqtt://192.168.1.100:1883` | `192.168.1.100` | `1883` | `False` |
| `mqtts://broker.host.com:8883` | `broker.host.com` | `8883` | `True` |
| `mqtt://host` | `host` | `1883` | `False` |
| `mqtts://host` | `host` | `8883` | `True` |
| `invalid` | — | — | `None` |

#### `update_broker_url` — lógica

```
1. _parse_broker_url(broker_url) == None? → return False  (formato inválido)
2. read_config() == None? → return False
3. adopted_status != ADOPTED? → return False  (device não adotado)
4. config["broker_url"] = broker_url
5. gravar config.json → return True
```

> Guarda `adopted_status == ADOPTED` garante que só devices ativos atualizam o broker.

#### `get_broker_params` — uso

Chamada pelo `mqtt_client.py` para obter `host`, `port` e `ssl` antes de conectar ao broker. Centraliza o parse da URL em um único lugar.

**Onde `update_broker_url` é chamada:** `mqtt/mqtt_client.py`, no handler do tópico `{topic}/config` quando `action == "RECONNECT"`.

---

## Estrutura final

```
embedded/esp/
├── config/
│   └── config.py        ✅ Implementado — config.json genérico
├── mqtt/
│   └── mqtt_config.py   ✅ Implementado — broker URL, parse, params
└── /config.json         # Criado em runtime na raiz (não versionado)
```

**Separação de responsabilidades:**

| Arquivo | Responsabilidade | Acessa broker? |
|---------|------------------|----------------|
| `config/config.py` | Ler/gravar `config.json`, MAC, adoção | ❌ Não |
| `mqtt/mqtt_config.py` | Atualizar `broker_url`, parsear URL, fornecer params de conexão | ✅ Sim |

---

## Decisões

| Questão | Decisão |
|---------|---------|
| **Async vs Sync** | Async — compatibilidade com loop principal (`uasyncio`) |
| **Criação do arquivo** | Criar na primeira execução se não existir |
| **Encoding** | `encoding="utf-8"` — padrão seguro |
| **Path do arquivo** | `/config.json` — path absoluto na raiz do filesystem |
| **Guarda de re-adoção** | Bloqueia apenas usuário diferente (idempotente para mesmo usuário) |
| **`device_uuid`** | Write-once: nunca sobrescrever após primeira adoção |
| **`wifi_password`** | Opcional no payload — permite redes abertas (string vazia válida); não persiste em `config.json` |
| **WiFi** | `wifi_ssid` e `wifi_password` residem em `/wifi.json` — gerenciados por `wifi/config_wifi.py` |
| **Helpers privados** | `_is_config_complete`, `_build_init_config` extraídos para clareza; validação em `tool/validate.py` |
