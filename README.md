# Check api status 🦀

Testar requisições em API.

## Arquivo de Configuração

Requisição simples: 
```yml
requests:
  coffe_api:
    name: sample_api
    request:
      url: https://api.sampleapis.com/coffee/hot
      method: GET
    expected_status: 200
    cron_expression: "0/10 * * * * *"
    system_notify: false
    notify_type: ANY
```

Requisição dependente da resposta de outra API:
```yml
requests:
  api1:
    name: POST_MESSAGE
    depends_on:
      name: GET_TOKEN
      header_fields:
        - auth.token
      body_fields:
        - field1.field2
        - test1.test2
      request:
        url: http://127.0.0.1:5000/auth/token
        method: GET
        headers:
          test: 123
          test2: 1234
    request:
      url: http://127.0.0.1:5000/message/555555555
      method: POST
      headers:
        authorization: "Bearer {{auth.token}}"
        content-type: "application/json"
      body:
        test: "test {{test1.test2}} test"
        test2: "{{field1.field2}}"
    expected_status: 201
    cron_expression: "0/10 * * * * *"
    system_notify: true
    notify_type: ERROR
```