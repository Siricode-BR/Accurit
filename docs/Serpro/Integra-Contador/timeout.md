---
origem_pdf: Timeout - Integra Contador.pdf
origem_url: https://apicenter.estaleiro.serpro.gov.br/documentacao/api-integra-conta... (truncado no PDF)
capturado_em: 2026-10-17
conversao: manual-assistida
aviso: Cópia de trabalho. Em divergência, vale o PDF.
---

# Política de Timeout

Padronizar o comportamento de timeout dos serviços da API, garantindo previsibilidade para os
clientes, transparência em cenários de lentidão e melhor experiência de uso.

Esta política visa:

- tornar explícitos os gargalos de latência por serviço;
- reduzir impactos operacionais no API Gateway;
- evitar suspensão de endpoints (`SUSPENDED`) por acionamento de circuit breaker;
- padronizar respostas de timeout com mensagem amigável.

Aplica-se a todas as integrações do Integra Contador identificadas por `IdSistema` e `IdServico`,
tendo como referência o Catálogo de Serviços.

Os valores de timeout por serviço são parâmetros operacionais e podem ser ajustados no
gerenciador do Integra Contador.

## Limites de tempo

O tempo máximo da resposta síncrona dessa API é de **30 segundos**, devido ao limite do API
Gateway.

Após esse limite, a chamada pode ser interrompida e contribuir para mecanismos de proteção
(circuit breaker), incluindo estado `SUSPENDED` do endpoint no gateway.

## Timeout por complexidade

- **Baixa complexidade:** 3 a 8 segundos
- **Média complexidade:** 10 a 20 segundos
- **Alta complexidade:** 20 a 28 segundos

Mesmo que o backend conclua após esse intervalo, o limite de resposta síncrona ao cliente
permanece restrito ao teto técnico do gateway (30s).

## Comportamento da API em timeout

Quando o serviço não responde dentro do tempo configurado, a API retorna:

- **HTTP Status:** `504 Gateway Timeout`
- **Código de erro:** `...058` <!-- conferir no PDF, p.2 -->
- **Mensagem:** `Não foi possível obter resposta do serviço {IdSistema}.{IdServico} no tempo esperado ({timeout} segundos). Tente novamente em instantes.`

## Semântica do 504 (estado indeterminado)

O retorno `504` indica que não houve resposta no tempo esperado pela API.

Isso **não garante** que o backend não concluiu a operação após o timeout.

Pode haver **persistência tardia** de dados na RFB, mesmo sem retorno do documento no corpo da
resposta.

## Regras importantes

Quando uma solicitação de envio retornar erro `504`:

1. Não envie a mesma solicitação novamente de forma imediata.
2. Considere que o processamento pode ter sido concluído, mesmo sem a resposta final.
3. Sempre que possível, consulte o status da solicitação antes de realizar um novo envio.
4. Reenvie apenas se for confirmado que a solicitação não foi recebida pela RFB.

Seguir essas orientações ajuda a evitar envios em duplicidade, geração de documentos repetidos
e correções desnecessárias posteriormente.

## Orientação para clientes

- Trate o erro `504` como uma falha temporária.
- Aguarde alguns instantes antes de tentar novamente.
- Consulte a situação da solicitação antes de reenviar quando possível.
- Guarde o `responseId` para facilitar o atendimento pelo suporte técnico, quando disponível.

## Aspecto comercial

Em caso de `504` por timeout, a chamada é tratada como não tarifada, conforme regra comercial
vigente.

---

Documento datado de: **8 de setembro de 2026**
