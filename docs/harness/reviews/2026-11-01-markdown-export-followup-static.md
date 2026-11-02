# Revisão estática delimitada — follow-up export

Reviewer: agente nativo `/root/review_projection_plan_v2`; mesma família, sem alegação de
independência entre modelos. Registro pelo integrador das duas respostas recebidas nesta sessão.
Somente leitura, sem execução do alvo, builds, testes ou rede. Não é max/post-gate/receipt humano.

## Rodada 1

- Base: `4bf79f85df7adaa9385eb8b9e459650d13fcc774`.
- Capture: `/private/tmp/engram-export-followup-review-a3nsnnae`.
- Target: `0a2d0f4d46e065eda7540f8588f355d475e9633f9cff473c8abbdda3b2d94855`.
- Manifest: `109721a87a6180229d10c49d919e3ec7c978978655de09d8b1da7d483b329005`.
- 11/11 paths e hashes reconciliados. Um achado MED bloqueante: prefixo `~e1-` case-sensitive
  permite literal `~E1-` aliasar o grupo codificado em APFS case-insensitive. IDs preservam folhas
  distintas; não foi alegado overwrite nem escape. Nenhum outro defeito concreto no delta.
- Durabilidade conferida estaticamente contra o SDK local macOS fcntl(2); custos/testes não atestados.

## Rodada 2

- Capture: `/private/tmp/engram-export-followup-review-v2-cn2hm1n4`.
- Target: `36482dffc930d58f4ae92e9df76012e663a3387b624ad49d359e1384b1dc3516`.
- Manifest: `1fc1461496327562b4361db1be433e251096784db30eefa349c833c152e947e0`.
- 11/11 hashes conferidos; só format.rs e naming_tests.rs diferem da rodada 1.
- Resposta: "O achado está resolvido na fonte. Não identifiquei novo defeito concreto neste delta."
  Prefixo compara bytes ASCII sem cortar UTF-8; teste cobre literal uppercase. Sem executar o teste.

Os captures têm base/candidate/trusted, target e manifest externos ao checkout; hashes do candidato
serão conferidos contra o commit. Documentos vivos do harness/risco não estavam nesses 11 paths e
não recebem aprovação implícita deste parecer. Gate completo e receipt humano são etapas distintas.
