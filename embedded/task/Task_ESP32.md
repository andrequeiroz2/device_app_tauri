# Task: ESP32 — Scripts Restantes

## Status Geral do Projeto

| Módulo | Arquivo | Status |
|--------|---------|--------|
| Config | `config/config.py` | ✅ Done |
| MQTT Config | `mqtt/mqtt_config.py` | ✅ Done |
| WiFi | `wifi/config_wifi.py` | ✅ Done |
| Tools | `tool/` (7 módulos) | ✅ Done |
| **Serial Handler** | `protocol/serial_handler.py` | ✅ Done |
| **Commands** | `protocol/commands.py` | ✅ Done |
| **MQTT Client** | `mqtt/mqtt_client.py` | ✅ Done |
| **Sensor DHT** | `sensors/dht.py` | ✅ Done |
| **Boot** | `boot.py` | ✅ Done |
| **Main** | `main.py` | ✅ Done |

---

## Achados da Documentação MicroPython ESP32

Análise de: https://docs.micropython.org/en/latest/esp32/quickref.html

### ⚠️ WLAN — Reconexão infinita (impacta `config_wifi.py`)

> "After a call to `wlan.connect()`, the device will by default retry to connect **forever**"

O `connect()` atual em `config_wifi.py` faz polling com timeout, mas o WLAN tenta reconectar em background indefinidamente. É necessário adicionar antes do `wlan.connect()`:

```python
wlan.config(reconnects=1)  # tenta uma vez; nosso loop controla o retry
```

> ⚠️ **Correção pendente** em `wifi/config_wifi.py` antes de testar em hardware.

---

### UART — Comunicação serial com o desktop

```python
from machine import UART
uart = UART(0, baudrate=115200)  # UART0: GPIO1=TX, GPIO3=RX (USB-CDC)
```

- **UART0** é o canal USB-CDC — é o que o app Tauri usa via `/dev/ttyUSB0` ou similar
- `uart.read()` e `uart.readline()` para leitura
- `uart.write(b"...\n")` para resposta
- Usar `select.poll()` para leitura não bloqueante (mencionado no ROLE.md como obrigatório)

---

### DHT — Sensor de temperatura e umidade

```python
import dht
from machine import Pin

d = dht.DHT11(Pin(4))
d.measure()           # dispara leitura — blocante ~15ms
d.temperature()       # int, °C
d.humidity()          # int, %RH
```

- Módulo `dht` é built-in no MicroPython para ESP32
- `measure()` deve ser chamado antes de cada leitura
- DHT11: temperatura inteira; DHT22: float com 1 casa decimal
- Intervalo mínimo entre leituras: ~1s (DHT11), ~2s (DHT22)

---

### Timer — Leituras periódicas de sensor

```python
from machine import Timer

tim = Timer(0)
tim.init(period=5000, mode=Timer.PERIODIC, callback=lambda t: read_sensor())
```

- ESP32 tem 4 timers de hardware (id 0–3)
- Callbacks são soft interrupts — sem acesso a hardware complexo dentro do callback
- Usar flag + polling no loop principal ao invés de lógica pesada no callback

---

### WDT — Watchdog timer

```python
from machine import WDT

wdt = WDT(timeout=8000)  # 8 segundos
wdt.feed()               # resetar o timer — chamar no loop principal
```

- Reinicia o device se o loop principal travar
- `wdt.feed()` deve ser chamado em cada ciclo do `main.py`

---

### RTC — Timestamps para MQTT

```python
from machine import RTC

rtc = RTC()
rtc.datetime()  # (year, month, day, weekday, hour, minute, second, subsecond)
```

- Usado para montar `"timestamp"` nos payloads MQTT
- Sem NTP o clock começa do zero no boot — considerar sincronização NTP após conectar

---

### select.poll — Leitura não bloqueante da UART

```python
import select

poll = select.poll()
poll.register(uart, select.POLLIN)

events = poll.poll(0)     # 0 = não bloqueante
if events:
    line = uart.readline()
```

- Obrigatório pelo ROLE.md para não travar o loop principal
- `poll(0)` retorna imediatamente; `poll(timeout_ms)` aguarda com timeout

---

## Planejamento dos Scripts

### 1. `protocol/serial_handler.py`

**Responsabilidade:** leitura de linhas JSON da UART, parse, despacho para `commands.py`.

**API planejada:**

| Função | Tipo | Descrição |
|--------|------|-----------|
| `init()` | sync | Inicializa `UART(0, 115200)` e `select.poll` |
| `poll()` | sync | Verifica se há dados na UART sem bloquear; retorna linha ou `None` |
| `handle(line)` | sync | Parseia JSON, roteia para handler correto, serializa resposta |

**Fluxo:**

```
1. uart = UART(0, 115200)
2. poll = select.poll(); poll.register(uart, POLLIN)
3. poll() no loop principal:
   - eventos? → uart.readline() → handle(line)
   - sem eventos? → retorna None imediatamente
4. handle(line):
   - json.loads(line) → {"cmd": "...", "data": {...}}
   - cmd → dispatch para commands.py
   - json.dumps(response) + "\n" → uart.write()
5. Erros: OSError, ValueError → {"ok": false, "error": "..."} + "\n"
```

---

### 2. `protocol/commands.py`

**Responsabilidade:** handlers por comando. Retornam dict para serializar.

**Comandos a implementar (conforme ROLE.md):**

| Comando | Request | Ação | Response |
|---------|---------|------|----------|
| `ping` | `{"cmd":"ping","data":null}` | — | `{"ok":true,"version":"1.0.0"}` |
| `get_info` | `{"cmd":"get_info","data":null}` | `read_config()` | campos do device + `firmware_version` |
| `get_config` | `{"cmd":"get_config","data":null}` | `read_config()` | config.json completo |
| `set_config` | `{"cmd":"set_config","data":{...}}` | adoção completa | `{"ok":true}` ou `{"ok":false,"error":"..."}` |
| `reboot` | `{"cmd":"reboot","data":null}` | `machine.reset()` | `{"ok":true}` (envia antes de reiniciar) |

**`set_config` — fluxo atômico (coordenado aqui):**

```
1. validate_adoption_data(data)              → False? {"ok":false,"error":"..."}
2. add_wifi(data["wifi_ssid"], data["wifi_password"])
3. await connect(wifi_ssid, wifi_password)   → False? rollback + {"ok":false,...}
4. adopt_device(data)                        → False? rollback + {"ok":false,...}
5. {"ok": true}
```

**API planejada:**

| Função | Tipo | Descrição |
|--------|------|-----------|
| `handle_ping(data)` | sync | Retorna versão do firmware |
| `handle_get_info(data)` | sync | Retorna campos do device + `firmware_version` |
| `handle_get_config(data)` | sync | Retorna `config.json` completo |
| `handle_set_config(data)` | async | Fluxo de adoção atômica |
| `handle_reboot(data)` | sync | Responde e chama `machine.reset()` |

---

### 3. `mqtt/mqtt_client.py`

**Responsabilidade:** conectar ao broker, publicar dados/heartbeat, subscrever tópicos, handler RECONNECT.

**Biblioteca:** `umqtt.simple` (built-in no MicroPython ESP32)

```python
from umqtt.simple import MQTTClient
```

**API planejada:**

| Função | Tipo | Descrição |
|--------|------|-----------|
| `connect()` | async | Lê params via `get_broker_params()`, conecta ao broker, subscreve tópicos |
| `disconnect()` | sync | Desconecta do broker |
| `publish_data(payload)` | sync | Publica em `{topic}/data` |
| `publish_status(state)` | sync | Publica em `{topic}/status` (heartbeat) |
| `poll()` | sync | `client.check_msg()` — não bloqueante; processar mensagens pendentes |
| `_on_message(topic, msg)` | sync | Callback interno: roteia `{topic}/config` e `{topic}/command` |
| `_handle_config(msg)` | sync | Trata `{"action":"RECONNECT","broker_url":"..."}` via `update_broker_url()` |
| `_handle_command(msg)` | sync | Trata `{"action":"ON"}` / `{"action":"OFF"}` (atuadores) |

**Tópicos (conforme ROLE.md):**

| Tópico | Direção | Payload |
|--------|---------|---------|
| `{topic}/status` | Publish | `{"state":"online","timestamp":"..."}` |
| `{topic}/data` | Publish | `{"temperature":25,"humidity":60,"timestamp":"..."}` |
| `{topic}/config` | Subscribe | `{"action":"RECONNECT","broker_url":"..."}` |
| `{topic}/command` | Subscribe | `{"action":"ON"}` ou `{"action":"OFF"}` |

---

### 4. `sensors/dht.py`

**Responsabilidade:** wrapper do driver built-in `dht` com logging e validação.

**API planejada:**

| Função | Tipo | Descrição |
|--------|------|-----------|
| `init(pin)` | sync | Inicializa `dht.DHT11(Pin(pin))` |
| `read()` | sync | Chama `measure()`, retorna `{"temperature": t, "humidity": h}` ou `None` em erro |

**Atenção:**
- `d.measure()` é bloqueante (~15ms para DHT11)
- **Intervalo entre leituras: 10 segundos** — regra de negócio do projeto
- Intervalo mínimo do hardware: 1s (DHT11) — 10s está bem acima do limite
- Leitura via `Timer(period=10000)` com flag, processada no loop principal (não dentro do callback)

---

### 5. `boot.py`

**Responsabilidade:** inicialização early antes do `main.py`.

**Planejado:**

```
1. esp.osdebug(None)        # silenciar output de debug do vendor
2. gc.collect()             # liberar memória antes do main
3. machine.freq(240000000)  # frequência máxima (opcional, para performance)
```

---

### 6. `main.py`

**Responsabilidade:** orquestração do loop principal — modo Configuração ou Operação.

**Fluxo:**

```
BOOT:
  1. config_init()            # garante config.json
  2. wifi_init()              # garante wifi.json
  3. serial_handler.init()    # inicializa UART + poll

MODO CONFIGURAÇÃO (adopted_status == 0):
  Loop:
    - serial_handler.poll()   # verifica comandos USB
    - wdt.feed()

MODO OPERAÇÃO (adopted_status == 1):
  1. await connect_from_config()   # conecta WiFi
  2. await mqtt_client.connect()   # conecta broker
  Loop:
    - serial_handler.poll()        # ainda escuta serial (reboot, get_info)
    - mqtt_client.poll()           # processa mensagens MQTT
    - sensor tick (via flag de Timer a cada 10s) → read() → publish_data()
    - heartbeat tick → publish_status("online")
    - wdt.feed()
```

---

## Estrutura final completa

```
embedded/esp/
├── boot.py                  ✅ Done
├── main.py                  ✅ Done
├── config/
│   └── config.py            ✅ Done
├── mqtt/
│   ├── mqtt_config.py       ✅ Done
│   └── mqtt_client.py       ✅ Done
├── protocol/
│   ├── serial_handler.py    ✅ Done
│   └── commands.py          ✅ Done
├── sensors/
│   └── dht.py               ✅ Done
├── wifi/
│   └── config_wifi.py       ✅ Done
└── tool/
    ├── load.py              ✅ Done
    ├── log.py               ✅ Done
    ├── network.py           ✅ Done
    ├── read.py              ✅ Done
    ├── validate.py          ✅ Done
    ├── wifi.py              ✅ Done
    └── write.py             ✅ Done
```

---

## Ordem de implementação sugerida

| Ordem | Módulo | Motivo |
|-------|--------|--------|
| 1 | `protocol/commands.py` + `protocol/serial_handler.py` | Desbloqueia adoção via USB — pré-requisito de tudo |
| 2 | `sensors/dht.py` | Simples, independente, necessário para o MQTT |
| 3 | `mqtt/mqtt_client.py` | Depende de `config.py`, `mqtt_config.py`, `sensors/dht.py` |
| 4 | `boot.py` | Trivial |
| 5 | `main.py` | Integra tudo — implementar por último |

---

## Decisões

| Questão | Decisão |
|---------|---------|
| **Baud rate** | Fixo em `115200` no firmware. O app Tauri tem seleção de baud rate para compatibilidade com outros dispositivos — se a comunicação falhar, cabe ao usuário selecionar 115200 no app. |
| **Intervalo de leitura do sensor** | 10 segundos fixo via `Timer(period=10000)` |
| **Async vs Sync** | Serial e sensores: sync. WiFi/MQTT: async via `uasyncio` |

---

## Correções aplicadas antes do teste em hardware

| Arquivo | Problema | Status |
|---------|---------|--------|
| `wifi/config_wifi.py` | `wlan.connect()` tentava reconectar indefinidamente — polling com timeout não funcionava | ✅ Corrigido — `wlan.config(reconnects=1)` adicionado |
