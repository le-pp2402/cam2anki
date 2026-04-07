# AnkiConnect API Requests

This document explains how to call AnkiConnect APIs to retrieve model information and add notes to Anki.

## 1. Endpoint

By default, AnkiConnect listens at:

```text
http://127.0.0.1:8765
```

Every request is an HTTP POST with a JSON body.

## 2. Common Request Format

```json
{
  "action": "<action_name>",
  "version": 6,
  "params": {
    "...": "..."
  }
}
```

Where:

- `action`: the API action to call.
- `version`: protocol version (typically `6`).
- `params`: action-specific parameters (can be omitted if not required).

## 3. Common Response Format

```json
{
  "result": "...",
  "error": null
}
```

- `result`: returned data when the request succeeds.
- `error`: `null` if there is no error, otherwise an error message.

## 4. Request Examples

### 4.1 Get model list (`modelNames`)

Request body:

```json
{
  "action": "modelNames",
  "version": 6
}
```

Curl:

```bash
curl -X POST http://127.0.0.1:8765 \
  -H "Content-Type: application/json" \
  -d '{"action":"modelNames","version":6}'
```

### 4.2 Get model field names (`modelFieldNames`)

Request body:

```json
{
  "action": "modelFieldNames",
  "version": 6,
  "params": {
    "modelName": "Memrise (Lτ) Template v5.1"
  }
}
```

Example response:

```json
{
  "result": [
    "Learnable",
    "Definition",
    "Audio",
    "Mems",
    "Attributes",
    "Extra",
    "Extra 2",
    "Choices"
  ],
  "error": null
}
```

Curl:

```bash
curl -X POST http://127.0.0.1:8765 \
  -H "Content-Type: application/json" \
  -d '{
    "action":"modelFieldNames",
    "version":6,
    "params":{"modelName":"Memrise (Lτ) Template v5.1"}
  }'
```

### 4.3 Add note (`addNote`)

Request body:

```json
{
  "action": "addNote",
  "version": 6,
  "params": {
    "note": {
      "deckName": "Japanese",
      "modelName": "Vocabulary",
      "fields": {
        "Word": "行く",
        "Reading": "いく",
        "Meaning": "<b>to go</b>",
        "Example": "<div>学校へ行く。</div>"
      },
      "tags": ["jp"]
    }
  }
}
```

Curl:

```bash
curl -X POST http://127.0.0.1:8765 \
  -H "Content-Type: application/json" \
  -d '{
    "action":"addNote",
    "version":6,
    "params":{
      "note":{
        "deckName":"Japanese",
        "modelName":"Vocabulary",
        "fields":{
          "Word":"行く",
          "Reading":"いく",
          "Meaning":"<b>to go</b>",
          "Example":"<div>学校へ行く。</div>"
        },
        "tags":["jp"]
      }
    }
  }'
```

## 5. Notes
- Anki must be running when you call AnkiConnect APIs.
- The AnkiConnect add-on must be installed in Anki.
- Always check the `error` field in the response.