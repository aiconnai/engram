# Markdown portability — follow-up de revisão, 2026-11-01

Input do owner: regressão em tags do Engram, custo de fsync e falhas preexistentes no import.
Base export: `4bf79f85df7adaa9385eb8b9e459650d13fcc774`, branch `fix/markdown-export-safety`
inicialmente limpa. Bootstrap executado. Checkout principal/documentos do planejamento preservados.
Correção/revalidação locais; sem push/PR/merge/tag/publicação ou dados reais. Tag aguarda bloqueantes.

## Frentes e estado

- Export na branch atual: codificar componentes, corrigir decoder Rust, otimizar sincronização
  preservando flush final; sem nova API de projeção/manifest. Regressores e medição antes/depois.
- Import: análise estática read-only por `/root/plan_quality_evidence` concluída; risco próprio
  RISK-0011 a registrar/reproduzir. Candidato separado, sem misturar seu código ao export.
- Unsafe/Rustix: dependência direta da versão 1.1.3 já presente no lock, inicialmente para fsync
  explícito seguro. Cargo.lock mudou só a lista de dependências do core, sem trocar versões.
- ACL: política conservadora continua; mensagem/documentação explicarão recusa inclusive deny-only.
- Linux: Docker disponível, daemon Linux/arm64; imagens Rust cached serão identificadas e usadas
  em execução isolada sem credenciais/egress, com arquivos de entrada explicitamente selecionados.

## Evidência parcial

- `target/markdown-export-followup/red.log` do checkout principal: 77 PASS/3 FAIL. Falhas funcionais:
  tags internas, colisão/validade de componentes e decoder público que convertia erro estruturado em
  sucesso zero; nenhuma falha de fixture. Produção ainda era a base ao executar.
- `green.log`: 81 PASS, 1 ignored (probe manual sem threshold temporal no CI). Tags session:, category:,
  confidence:, file:, trace: e aux/CON aceitas com destinos seguros; prefixo de escape reservado.
- Probe manual 300 memórias/301 arquivos, 227562 bytes, três execuções: antes 2.388540/2.373637/2.397917 s;
  depois 0.057277/0.049472/0.055461 s. Logs `perf-before.log`/`perf-after.log`; mesma máquina, profile ci,
  dataset sintético e protocolo. Medianas 2.388540→0.055461 s (~43x), sem extrapolar para 100K.
- Probe de syscalls isoladas (100 arquivos): file F_FULLFSYNC ~0.402 s, file fsync ~0.016 s; só mover
  dir sync ao final mantendo full por arquivo não resolveu (~0.404 s). Nenhum dado real usado.

Dados sincronizam com POSIX fsync antes do rename; diretórios por grupo, mkdir sincroniza o pai;
flush forte final por dispositivo contém filas previamente fsync'd (SDK local macOS fcntl(2)).
Antes da conclusão o lote pode estar incompleto; erro de flush final continua erro, coberto por
fault injection. Não há atomicidade global. Uma representação por dispositivo evita fd por arquivo.

Análise import: não alegar ausência de proteção SQLite: refuse_active_sqlite_artifact cobre aliases
estáveis; há janela check/open. IDOR por engram_id e falta de raiz/mode estão no fluxo; confirmar
dinamicamente em fixtures antes da correção. Nuance: dispatcher restrito exige workspace explícito,
e override prevalece na criação; não alegar bypass remoto de frontmatter ainda não demonstrado.

Nova revisão e full gate pendentes; PASS/515 s anterior não cobre este delta.

## Continuação após retomada

- RISK-0010 reconciliado (mesmo bloco nos dois checkouts) e RISK-0011 registrado, ambos abertos.
- Review estático encontrou um MED no codec: alias com prefixo uppercase em APFS. Teste RED antes
  da correção em `review-codec-red.log`; prefixo agora compara ASCII sem distinção de caixa.
  `green-final.log`: 81 PASS/1 probe ignorado, 1.79 s. Review rodada 2 fecha o achado na fonte.
  [Registro](../reviews/2026-11-01-markdown-export-followup-static.md), 11 paths reconciliados.
- Linux nativo arm64 em Docker/OrbStack, sem emulação: suíte inicial 81 PASS/1 probe ignorado,
  incluindo default ACL. Invocação inicial usou toolchain fixado indisponível em imagem read-only;
  execução corrigida declara Rust 1.97.1 disponível (Mac usa 1.96.0). Integrações iniciais receberam
  nome de target incorreto; rodada corrigida no mesmo cache deve ser registrada sem ocultar a falha.
- Imagem base local cached ID 83cf2817eac0ceb04d29e085eef3714d20714606ca931a915d376a20f4458046;
  preparada com dependências SO antes de montar código; candidato executa com network=none,
  rootfs read-only, cap-drop ALL, no-new-privileges, limites CPU/memória/pids e apenas scratch próprio.
  Sem Docker socket/credenciais/config global ou dados reais dentro do container. Só crates públicos
  do lock e índice correspondente copiados; receipt/config próprio em target/.../linux-environment.json.

## Uso de IA e recursos

- Integrador `/root`: implementação, fixtures RED/GREEN, medição, Linux/CI e integração local.
- `/root/plan_quality_evidence`: análise estática do import concluída, read-only.
- `/root/review_projection_plan_v2`: revisão estática export em duas rodadas; um achado e correção.
- Via nativa já autorizada; modelo/effort solicitado na configuração anterior Sol/high; identidade,
  tokens e custo efetivos não expostos pela interface (desconhecidos, não zero). Sem CLI externo pago.
- `/root/plan_setup` mostra erro anterior "workspace is out of credits"; não usado nem trocado de
  conta/rota para contornar. Revisão atual na via nativa respondeu. Não alegar aprovação formal.
- Cargo pesado serializado entre Linux/macOS; documentos/revisão estática concorrem sem build.

## Candidato export revisado

Linux final (`linux-final.log`, Rust 1.97.1 arm64): 81 PASS/1 probe manual ignorado, 0.92 s;
canonical_journey 10 PASS, permission_modes_tests 9 PASS, portability_permissions_routing_tests
3 PASS. Exit do container 0. Nova captura de 11 paths corresponde ao target da rodada estática 2.
Mac `green-final.log`: 81 PASS/1 probe manual ignorado. Doctor PASS com aviso histórico de ausência
de post-gate da sprint; diff --check e fmt --check PASS. Commit local prepara o candidato; full
novo continua pendente, sem reaproveitar o PASS histórico de 515 s como validação deste delta.

## Gate final do export

Commit `9bca4efaeb07d9deff722ab50c407dfe77f02f63`, hook ativo fmt+clippy all-targets/all-features PASS.
Os 11 hashes do manifest revisado foram conferidos contra os blobs do commit: todos iguais.
`sensors.sh` full PASS, timestamp 2026-11-01T16:08:11Z, 459 s, sem exclusões. Receipt e log canônicos
atualizados na branch export; `target/markdown-export-followup/full.log` preservado. Sem falha instável
nesta rodada. Formal post-gate, receipt humano e integração continuam pendentes. Import branch
`fix/markdown-import-safety` parte do commit de produto; seu RED roda depois deste gate, serialmente.

## Inventário público e evidência final

O complemento dos comandos obrigatórios de scripts/ci.sh encontrou um delta de documentação que
sensors.sh não cobre: ENGRAM_EXPORT_BASE_DIR anunciada em MCP_TOOLS mas ausente do baseline.
Gerador oficial --write adicionou só essa env var + metadados de captura; comparação estrutural
com o baseline anterior comprovou ausência de outras alterações/supressões. Mesmo arquivo gerado
nas duas branches, base9bca ancestral comum, hash a3b1ad10cf4bd29d9e68336372cf56e7b48f7d84280959bb1636951212161eb8.
Review estático adicional conferiu hashes, fonte anterior intacta e somente esse path novo:
export target780732afce19557d02fffdb5691a1231fe13f5f3fb7db8a624bdd3336a172ff3 (12regulares);
import targetfabb19e15d33cd476527e59bd077d524461c12ae6bca16de1d21973e8b632154 (17regulares+lookup ausente).
Raw outputs dos reviews preservados em reviews/*export-followup-v*.txt, *import-v*.txt e
*markdown-contract-addendum.txt (somente os aplicáveis a esta branch). Não são approval/receipt.

Teste advertised-contracts nesta branch: 7 PASS após atualizar o baseline documental. Full459 s
continua evidência do código Rust idêntico a9bca; adendo não alterou código, só contrato e artefatos.
Não houve novo full para esse delta documental. O programa de evolução/checkout principal continua
preservado e não entra nestes commits. Import permanece em candidato separado.
