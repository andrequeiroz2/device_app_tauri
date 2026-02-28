# Task: Config (config.json)

## Objetivo
Implementar módulo de configuração persistente para o device embarcado, criando e gerenciando `config.json` conforme `embedded/ROLE.md`.

## Análise do Código de Referência

**Projeto analisado:** `/home/andre/RustroverProjects/device_app_micropython`

### Estrutura encontrada

| Arquivo | Responsabilidade |
|---------|------------------|
| `config_tool.py` | Classe `DeviceConfig` com métodos para config |
| `device_config.json` | Arquivo de persistência |

### Métodos em `config_tool.py`

| Método | Retorno | Descrição |
|--------|---------|-----------|
| `load_device_config()` | `bool` | Verifica se arquivo existe via `os.stat()` |
| `read_device_config()` | `dict \| None` | Abre arquivo, `json.load(f)`, retorna `None` em erro |
| `get_mac_address()` | `str \| None` | Obtém MAC via `network.WLAN(network.STA_IF).config('mac')` |
| `write_init_config()` | `bool` | Cria config inicial se arquivo não existe ou está incompleto |
| `write_adopted_device(...)` | `dict` | Atualiza config com dados da adoção |

### Lógica de `write_init_config` (referência)

1. Verifica se arquivo existe (`load_device_config`)
2. Se não existe → erro e retorna False
3. Lê config (`read_device_config`)
4. Se config já tem todos os campos obrigatórios → "already registered", retorna True
5. Caso contrário: monta config com defaults + `get_mac_address()`, grava em disco

### Lógica de `write_adopted_device` (referência)

1. Valida tipos (device_name, user_uuid, device_uuid, broker_url, topic são str)
2. Lê config atual
3. Se `adopted_status != 0` → erro 409 "already adopted"
4. Se `user_uuid` ou `device_uuid` já preenchidos → erro 409
5. Atualiza campos no dict, grava em disco

### Diferenças para nosso projeto

| Aspecto | device_app_micropython | Nosso embedded (ROLE) |
|---------|------------------------|------------------------|
| Arquivo | `device_config.json` | `config.json` |
| WiFi | Separado em `wan_config.json` | Incluído em config (wifi_ssid, wifi_password) |
| Adopted | `write_adopted_device(...)` 5 params | `set_config` envia objeto completo |
| device_scale | Typo "humidty" | "humidity" (ROLE) |
| Async | Todos métodos `async` | Async (decisão) |

---

## Fases de Implementação

### Fase 1: Constantes e estrutura base

**Arquivo:** `embedded/esp/config/config.py`

- Definir `CONFIG_FILE = "/config.json"` (path absoluto, raiz do filesystem)
- Definir constantes de default:
  - `BOARDER_TYPE = "ESP32"`
  - `DEVICE_TYPE = "Sensor"` (ou "Actuator")
  - `SENSOR_TYPE = "DHT11"` (ou "" para actuator)
  - `ACTUATOR_TYPE = ""` (ou "relay" para actuator)
  - `DEVICE_SCALE = [["temperature", "C"], ["humidity", "%"]]`
  - `ADOPTED_STATUS = 0`
  - `ADOPTED_STATUS_DESC = "not_adopted"`
- Classe/enum `AdoptedStatus`: `UNADOPTED = 0`, `ADOPTED = 1`, `DESC = {0: "not_adopted", 1: "adopted"}`

**Entregável:** Constantes e enum definidos.

---

### Fase 2: Verificar existência do arquivo

**Método:** `async def load_config() -> bool`

- Usar `os.stat(CONFIG_FILE)` para verificar se arquivo existe
- Em `OSError`: logar erro, retornar `False`
- Sucesso: retornar `True`

**Referência:** `DeviceConfig.load_device_config()` em config_tool.py (linhas 34-40)

**Entregável:** Função que retorna True/False conforme arquivo existe.

---

### Fase 3: Ler configuração

**Método:** `async def read_config() -> dict | None`

- Abrir arquivo com `open(CONFIG_FILE, "r", encoding="utf-8")`
- Usar `json.load(f)` para parsear
- Em exceção: retornar `None` (arquivo inválido ou inexistente)
- Fechar arquivo corretamente (ou usar `with`)

**Referência:** `DeviceConfig.read_device_config()` (linhas 42-48)

**Entregável:** Função que retorna dict da config ou None.

---

### Fase 4: Obter MAC address

**Método:** `async def get_mac_address() -> str | None`

- Ativar interface: `wlan = network.WLAN(network.STA_IF)` e `wlan.active(True)` se necessário
- Obter MAC: `wlan.config('mac')` → bytes
- Formatar: `':'.join('{:02X}'.format(b) for b in mac)` (uppercase)
- Retornar string no formato `"3C:71:BF:4D:DB:0C"`
- Em exceção: retornar `None`

**Referência:** `DeviceConfig.get_mac_address()` (linhas 51-59)

**Entregável:** Função que retorna MAC no formato esperado pelo backend.

---

### Fase 5: Escrever config inicial

**Método:** `async def write_init_config() -> bool`

1. Se arquivo não existe (`load_config()` retorna False) → obter MAC, montar config inicial, gravar em disco e retornar True
2. Caso exista: chamar `read_config()`
3. Se config tem todos os campos obrigatórios (conforme ROLE) → retornar True
4. Caso incompleto:
   - Obter MAC via `get_mac_address()`; se None → retornar False
   - Montar dict com defaults: `adopted_status`, `adopted_status_desc`, `device_type`, `sensor_type`, `actuator_type`, `boarder_type`, `mac_address`, `device_scale`, `broker_url`, `topic`, `user_uuid`, `device_uuid`, `device_name`, `wifi_ssid`, `wifi_password` (strings vazias para campos de adoção)
   - Gravar com `json.dump(config, f)` em `open(CONFIG_FILE, "w", encoding="utf-8")`

**Campos obrigatórios para considerar "already registered":**  
boarder_type, mac_address, device_type, sensor_type, actuator_type, adopted_status, adopted_status_desc, device_scale (e demais conforme ROLE).

**Entregável:** Config inicial criada quando necessário.

---

### Fase 6: Atualizar config (adoção / set_config)

**Método:** `async def update_config(data: dict) -> bool`

Recebe o payload de `set_config` (user_uuid, device_uuid, device_name, topic, broker_url, wifi_ssid, wifi_password, adopted_status, adopted_status_desc).

1. Validar tipos (strings não vazias para campos obrigatórios)
2. Ler config atual com `read_config()`; se None → retornar False
3. Se `adopted_status == 1` e config já tem user_uuid/device_uuid → retornar False (já adotado)
4. Atualizar campos no dict: device_name, user_uuid, device_uuid, topic, broker_url, wifi_ssid, wifi_password, adopted_status, adopted_status_desc
5. Gravar com `json.dump(config, f)` em `open(CONFIG_FILE, "w", encoding="utf-8")`
6. Retornar True em sucesso

**Referência:** `write_adopted_device` (linhas 148-212), mas incluindo wifi_ssid e wifi_password.

**Entregável:** Config atualizada após set_config.

---

### Fase 7: Atualizar broker_url (troca de broker via MQTT)

**Método:** `async def update_broker_url(broker_url: str) -> bool`

Para o fluxo de "Device (app) vs Device (físico)" – quando o usuário troca o broker no app.

1. Validar broker_url (string não vazia)
2. Ler config atual
3. Atualizar apenas `broker_url`
4. Gravar e retornar True/False

**Entregável:** Suporte à troca de broker sem reconfiguração USB.

---

## Estrutura final esperada

```
embedded/esp/
├── config/
│   └── config.py    # Todas as funções acima
└── /config.json     # Criado em runtime na raiz (não versionado)
```

## Decisões

| Questão | Decisão |
|---------|---------|
| **Async vs Sync** | Async – seguir padrão do projeto de referência e compatibilidade com loop principal |
| **Criação do arquivo** | Criar na primeira execução se não existir |
| **Encoding** | `encoding="utf-8"` – padrão seguro e compatível com nomes/accentos |
| **Path do arquivo** | `/config.json` – path absoluto na raiz do filesystem, alinhado ao referência (`/device_config.json`) |
