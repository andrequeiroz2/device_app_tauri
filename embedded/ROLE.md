# Role de Engenharia (Embedded / MicroPython)

## Título e Escopo
Arquiteto de aplicações embarcadas, com experiência em MicroPython, responsável por firmware para ESP32/ESP8266, protocolo de provisionamento via serial USB, integração com o desktop app (Tauri) e boas práticas de código embarcado.

## Responsabilidades
- **DEVE** manter arquitetura limpa e modular, separando protocolo serial, configuração persistente e lógica de sensores/atuadores.
- **DEVE** garantir compatibilidade com o protocolo de adoção definido pelo backend (`src-tauri/api/device/provisioning/protocol.rs`).
- **DEVE** otimizar uso de memória (RAM/Flash) e evitar alocações dinâmicas desnecessárias em runtime.
- **DEVE** usar MicroPython como linguagem principal para prototipagem rápida e manutenção simples.
- **DEVE** documentar formato de comandos/respostas JSON e fluxo de provisionamento.
- **DEVE** tratar timeouts e erros de comunicação serial de forma robusta.

## Princípio: Device Inteligente

O device **já sabe quem é** (type, mac, model, scales). O app desktop apenas:
1. Lê informações do device (`get_info`)
2. Completa com dados do sistema (`set_config`: user_uuid, topic, broker, WiFi)
3. Reinicia o device (`reboot`)

**Não é necessário transferir firmware** – o device já vem pronto.

---

## Stack

**Plataforma:**
- ESP32, ESP8266, Raspberry Pi Pico, Pyboard, STM32 (qualquer placa com MicroPython)
- MicroPython (versão estável recomendada)

**Comunicação:**
- Serial UART (USB-CDC ou USB-serial): 115200 baud default, 8N1.
- MQTT (após adoção): para telemetria, comandos e heartbeat.

**Persistência:**
- Arquivo `config.json` no sistema de arquivos MicroPython.

---

## Modos de Operação

| Modo | Condição | Comportamento |
|------|----------|---------------|
| **Configuração** | `adopted_status == 0` | Escuta comandos JSON na Serial |
| **Operação** | `adopted_status == 1` | Conecta WiFi → MQTT, publica dados/heartbeat |

---

## Estrutura config.json

### Antes da adoção (get_info)

```json
{
  "adopted_status": 0,
  "adopted_status_desc": "not_adopted",
  "device_type": "Sensor",
  "sensor_type": "DHT11",
  "actuator_type": "",
  "boarder_type": "ESP32",
  "mac_address": "3C:71:BF:4D:DB:0C",
  "device_scale": [["temperature", "C"], ["humidity", "%"]]
}
```

### Após adoção (set_config)

Campos adicionais gravados pelo `set_config`:

```json
{
  "adopted_status": 1,
  "adopted_status_desc": "adopted",
  "user_uuid": "a8b16f77-abe2-4c08-bed1-0236bea80561",
  "device_uuid": "a6f53396-eab9-432d-88b0-e1154fa5e551",
  "device_name": "Sala Sensor",
  "topic": "a8b16f77-abe2-4c08-bed1-0236bea80561/a6f53396-eab9-432d-88b0-e1154fa5e551",
  "broker_url": "mqtt://192.168.1.100:1883",
  "wifi_ssid": "MinhaRede",
  "wifi_password": "senha123"
}
```

> **broker_url**: Formato `mqtt://host:port` ou `mqtts://host:port`. Parsar para obter host e port do MQTT.

### Campos obrigatórios (config.json)

| Campo | Origem | Descrição |
|-------|--------|-----------|
| `adopted_status` | Device | `0` = não adotado, `1` = adotado |
| `adopted_status_desc` | Device | `"not_adopted"` ou `"adopted"` |
| `device_type` | Device | `"Sensor"` ou `"Actuator"` |
| `sensor_type` | Device | `"DHT11"`, `"BME280"`, etc. (sensores) |
| `actuator_type` | Device | `"relay"`, `"motor"`, etc. (atuadores) |
| `boarder_type` | Device | `"ESP32"`, `"RP2"`, `"Pyboard"` |
| `mac_address` | Device | Identificador único do hardware |
| `device_scale` | Device | `[["measurement","unit"],...]` para sensores |
| `user_uuid` | set_config | UUID do usuário dono |
| `device_uuid` | set_config | UUID do device |
| `device_name` | set_config | Nome dado pelo usuário |
| `topic` | set_config | Base MQTT: `{user_uuid}/{device_uuid}` |
| `broker_url` | set_config | URL do broker MQTT |
| `wifi_ssid` | set_config | SSID da rede WiFi |
| `wifi_password` | set_config | Senha WiFi |

### device_scale – Escalas de medição (sensores)

Formato: `[["measurement", "unit"], ...]`

| Sensor | device_scale |
|--------|--------------|
| DHT11/DHT22 | `[["temperature","C"],["humidity","%"]]` |
| BMP280 | `[["temperature","C"],["pressure","hPa"]]` |
| DS18B20 | `[["temperature","C"]]` |
| MQ-135 | `[["air_quality","ppm"]]` |
| LDR | `[["luminosity","lux"]]` |

---

## Protocolo Serial JSON

**Formato:** Uma linha JSON por comando/resposta, terminada em `\n`. Responder em até 5 segundos (timeout do desktop app).

### Comandos suportados

| Comando | Request | Response |
|---------|---------|----------|
| `ping` | `{"cmd":"ping","data":null}` | `{"ok":true,"version":"1.0.0"}` |
| `get_info` | `{"cmd":"get_info","data":null}` | Ver estrutura abaixo |
| `get_config` | `{"cmd":"get_config","data":null}` | `{...config.json completo...}` |
| `set_config` | `{"cmd":"set_config","data":{...}}` | `{"ok":true}` |
| `reboot` | `{"cmd":"reboot","data":null}` | `{"ok":true}` (e reinicia) |

### Resposta de get_info (obrigatória)

```json
{
  "ok": true,
  "adopted_status": 0,
  "device_type": "Sensor",
  "sensor_type": "DHT11",
  "actuator_type": "",
  "boarder_type": "ESP32",
  "mac_address": "3C:71:BF:4D:DB:0C",
  "device_scale": [["temperature","C"],["humidity","%"]],
  "firmware_version": "1.0.0"
}
```

> `adopted_status`: `0` = livre para adoção, `1` = já adotado (bloqueia nova adoção até reset).

### Payload de set_config (enviado pelo Tauri)

```json
{
  "user_uuid": "a8b16f77-abe2-...",
  "device_uuid": "a6f53396-eab9-...",
  "device_name": "Sala Sensor",
  "topic": "a8b16f77.../a6f53396...",
  "broker_url": "mqtt://192.168.1.100:1883",
  "wifi_ssid": "MinhaRede",
  "wifi_password": "senha123",
  "adopted_status": 1,
  "adopted_status_desc": "adopted"
}
```

---

## Tópicos MQTT (após adoção)

**Formato base:** `{user_uuid}/{device_uuid}/{tipo}`

| Tipo | Tópico | Direção | Payload |
|------|--------|---------|---------|
| Heartbeat/Status | `{topic}/status` | Publish | `{"state":"online","timestamp":"..."}` ou `{"state":"offline"}` |
| Dados (sensor) | `{topic}/data` | Publish | `{"temperature":25.5,"humidity":60,"timestamp":"..."}` |
| Dados (actuator, futuro) | `{topic}/data` | Publish | Ex.: `{"setpoint":25}` ou `{"output":40}` — estado/setpoint do atuador |
| Comando (actuator) | `{topic}/command` | Subscribe | Varia por `actuator_type`; device recebe |

**Exemplo:** `a8b16f77-abe2-.../a6f53396-eab9-.../status`

**Status vs Comando:**
- `/status` indica **conectividade** (`online` ou `offline`) — igual para sensor e atuador. O device **não** publica ON/OFF em `/status`.
- `/command` contém **comandos** enviados pelo app ao device. O formato depende do `actuator_type`:
  - **ON/OFF (relay, switch):** `{"action":"ON"}` ou `{"action":"OFF"}`
  - **Valores numéricos (futuro, ex. termostato, válvula):** `{"action":"set_temp","value":25}` ou `{"value":40}` — estruturas extensíveis
- `/data` (actuadores): Atuadores ON/OFF hoje não publicam em `/data`. Atuadores futuros (termostato, dimmer etc.) podem publicar estado/setpoint, ex.: `{"setpoint":25}` (5°C, 40°C etc.), para o dashboard exibir o valor atual

### LWT (Last Will and Testament)

O device **DEVE** configurar LWT ao conectar para que, em perda inesperada de conexão (queda de energia, rede, crash), o broker publique automaticamente o estado offline. Sem LWT, o `operation_status` permanece `online` indefinidamente.

| Parâmetro | Valor | Descrição |
|-----------|-------|-----------|
| Tópico | `{topic}/status` | Mesmo do heartbeat |
| Payload | `{"state":"offline"}` | Compatível com status_processor do collector |
| QoS | 1 | Recomendado |
| Retain | true | Última mensagem reflete o estado real |

O LWT só é publicado pelo broker quando o cliente é declarado morto (ex.: ausência de PINGREQ dentro do keep-alive). Em desconexão graciosa (`disconnect()`), o device deve publicar `{"state":"offline"}` manualmente antes de desconectar.

**Validação:** Executar `scripts/simulate_esp32_device.py`, matar com `kill -9 <pid>`, verificar no app que `operation_status` passa para `offline`.

**Nota sobre set_config:** O payload de adoção não inclui LWT. O firmware usa sempre o payload fixo `{"state":"offline"}` no LWT, compatível com o status_processor do collector. O device no banco pode ter `lwt_enabled`, `lwt_message` etc. (para exibição/UI), mas o firmware não os lê — usa LWT fixo por simplicidade.

---

## Device (app) vs Device (físico)

**Não confundir:**
- **Device (app Tauri)**: Registro no banco com `broker_uuid`, configurações MQTT, etc.
- **Device (físico)**: Placa ESP32 que precisa conhecer **broker host:port** e **tópicos** para conectar.

O formato dos tópicos (`{user_uuid}/{device_uuid}/...`) é independente do broker. Mas o device físico **DEVE** saber o broker atual para conectar.

### Troca de broker

Quando o usuário altera o broker no app, os devices físicos **DEVEM** ser informados via MQTT para reconectar ao novo broker:

1. App publica no broker atual (antes da troca) comando de reconexão.
2. Device recebe (ex.: em `{topic}/config` ou `{topic}/command`) payload como: `{"action":"RECONNECT","broker_url":"mqtt://novo-host:1883"}`.
3. Device desconecta do broker atual, atualiza `config.json` com o novo `broker_url`, conecta ao novo broker.
4. Continua publicando nos mesmos tópicos (estrutura permanece).

O firmware **DEVE** implementar handler para esse comando e tratar a reconexão sem exigir USB ou reconfiguração manual.

---

## Reset / Re-adoção

Para permitir que o device seja adotado por outro usuário:

1. **No device:** Botão de reset ou comando especial → apaga `config.json` ou seta `adopted_status: 0`
2. **No banco:** Soft delete (`is_active = false`) libera o MAC para nova adoção
3. **Novo usuário:** Executa fluxo de adoção normalmente

---

## Padrões Arquiteturais (OBRIGATÓRIOS)

### Estrutura de Módulos – DEVE seguir

```
embedded/esp/
├── ROLE.md              # Este arquivo
├── main.py              # Entry point, loop principal
├── boot.py              # Inicialização early (se necessário)
├── config/
│   └── config.py        # Carregar/salvar config.json
├── protocol/
│   ├── serial_handler.py   # Leitura de linhas JSON, despacho
│   └── commands.py        # Handlers: ping, get_info, set_config, reboot
├── mqtt/
│   └── mqtt_client.py   # Cliente MQTT (conectar, publicar, subscrever)
├── sensors/             # (opcional) drivers de sensores
└── actuators/           # (opcional) drivers de atuadores
```

### Separação de Responsabilidades – OBRIGATÓRIO

- **`serial_handler`**: Leitura de linhas, parse JSON, roteamento. **NUNCA** lógica pesada.
- **`commands`**: Handlers por comando. Retornam dict para serializar em JSON.
- **`config`**: Carregar/salvar `config.json`. **NUNCA** acessar hardware além de storage.
- **`main.py`**: Orquestração do loop (serial, MQTT, sensores).

### Comunicação Serial – OBRIGATÓRIO

- **SEMPRE** ler até `\n` ou timeout.
- **SEMPRE** responder com `json.dumps(obj) + "\n"`.
- **SEMPRE** tratar `OSError`, `ValueError` e timeout sem travar o loop.
- Baud rate configurável (default 115200).

### Memória e Performance – OBRIGATÓRIO

- Evitar loops bloqueantes longos; usar `select.poll` ou `machine.idle()` ao esperar entrada.
- Preferir strings e buffers fixos onde possível.
- **NUNCA** concatenar strings em laços longos; usar `bytes`/`bytearray` para buffers.

### Segurança – OBRIGATÓRIO

- **NUNCA** logar senhas ou credenciais.
- **SEMPRE** validar tamanho e formato de JSON antes de processar.
- Credenciais **SOMENTE** após `set_config` validado.

---

## Padrões e Guias (OBRIGATÓRIOS)

**Nomenclatura – OBRIGATÓRIO:**
- Textos, comentários e variáveis **SEMPRE** em inglês.
- Arquivos: `snake_case`. Constantes: `UPPER_SNAKE_CASE`.

**Código MicroPython – OBRIGATÓRIO:**
- Módulos pequenos, responsabilidade única.
- Docstrings em funções públicas.
- **SEMPRE** tratar exceções em handlers; retornar `{"ok":false,"error":"..."}` em falhas.

**Integração com Desktop – OBRIGATÓRIO:**
- Protocolo **DEVE** ser compatível com `src-tauri/api/device/provisioning/protocol.rs`.
- Campos JSON **DEVEM** coincidir com o backend.
- Testar adoção com o wizard do app Tauri antes de considerar concluído.

**Build e Deploy – OBRIGATÓRIO:**
- **SEMPRE** documentar passos de flash (esptool, mpremote, Thonny).
- **SEMPRE** versionar `firmware_version` no `get_info`.
- Manter `README.md` em `embedded/esp/` com setup e requisitos.
