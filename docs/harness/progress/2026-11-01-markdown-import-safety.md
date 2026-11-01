# Markdown import — RISK-0011, 2026-11-01

Base `9bca4efa` (export follow-up local). Branch/worktree próprias. Bootstrap executado; arquivos de
política obrigatórios são os mesmos blobs já lidos no export, SPEC ganha só esta exceção delimitada.
Escopo e contrato no canvas. Tag aguarda bloqueantes; sem push/PR/merge/tag/release, produção,
dados reais, instalação de clientes ou inferência externa paga. Risco continua aberto até integração.

## Fila e ownership

- `/root/plan_quality_evidence`: escrever apenas `import/security_tests.rs`, contra API existente,
  sem builds/execução do alvo, rede ou filhos. Testes serão executados pelo integrador para RED.
- `/root`: mod wiring, reader seguro, permissões, transações/handlers, catálogo/docs, execução serial
  de Cargo macOS/Linux e integração. Sem alterar políticas/scripts de gates.
- Review estático posterior em captura congelada; formal post-gate/receipt humano continuam distintos.
- Cargo export full rodando em worktree export; import só recebe leitura/testes/documentos por ora.

## Aceite

Reproduzir IDOR/leitura por links e ausência de raiz antes da correção; confirmar negação sem efeitos
e controles positivos próprios após. Validar limites de reader, artefatos SQLite antes do open,
Atomicidade do lote DB e autorização/reclassificação no mesmo snapshot/transação; dry-run legado,
roundtrip/tag/versão, wrapper de erros, Linux nativo e gates no candidato correto.

Modelo/effort solicitado na via nativa anterior Sol/high; identidade efetiva/tokens/custo não expostos.
Sem alegar independência entre modelos ou custo zero. Nenhuma troca de conta/rota por crédito.

## RED independente da implementação

`import-red.log`: 14 testes, 2 PASS (helper/control positivo), 12 FAIL, 10.40 s; compilação sem erros.
IDOR confirm retornou applied=1 no ID estrangeiro; lote próprio+estrangeiro retornou applied=2;
symlink de diretório trouxe arquivo externo para memória. FIFO bloqueou e foi morto pelo timeout
privado de 10 s. Demais falhas cobrem preview/force, hardlink/ciclo/IDs inválidos/mode antes de IO.
Fontes e hashes antes da correção em target/markdown-export-followup/import-red-source, após o gate
export. Só então o draft de scratch foi aplicado à produção. Nenhum dado real foi acessado.

## GREEN focal e cobertura

`import-units-final.log`: 98 PASS/1 probe manual ignorado, 1.71 s, features obrigatórias do CI.
`import-integrations-final.log` (sem filtro de nome): routing3 PASS, G1 8 PASS/1 helper ignorado
(o helper é executado pelos testes pais), vault5 PASS. O novo G1 testa symlinks e hardlinks do
DB/WAL/SHM; após cada recusa, outro processo abre/fecha e commits continuam visíveis ao reinício.
A falha injetada no segundo update comprova rollback do primeiro, inclusive versão/tags/importância.

Primeira tentativa GREEN não compilou: API Storage não tem with_read_connection; snapshot review
usa with_connection + transação deferred somente leitura. Corrigido junto do import não usado.
Uma invocação focal aplicou filtro Markdown também às integrações (zero executadas); não foi aceita
como evidência. Comando separado sem filtro executou as integrações listadas acima.

Captura estática: target 73ef357922375f266d77ce3bdda295b463823604aebe01adbd228b86d2d802c8,
manifest b1be35910a84c399bfc5bb052d2b16a6db52d8a8be5b6b3b802312326d2544ab; 17 paths, incluindo
lookup.rs removido (type=absent). `/root/review_projection_plan_v2` analisa read-only, sem executar.
Ainda faltam validação Linux atual e full no import; PASS export não vale para seu delta.

## Primeira revisão estática e correções

17 paths conferidos. Dois bloqueantes: MED parser perdia campos com tab; HIGH applied0 incondicional
em erro pós-commit (Storage revalida permissões depois de tx.commit()). Ambos aceitos.

- Teste tab RED (`import-review-red-2.log`) mostrou criação indevida applied1 com engram_id tab0.
- Fixture pós-commit inicial não compilou por StorageConfig sem Default; corrigida. A primeira
  ideia -journal não reproduziu (hardening só inclui DB/WAL/SHM), não conta como prova do achado.
- Fixture corrigida usa alias de pai configurado: SQLite abre pai canônico; trocar só esse alias por
  destino sintético faz hardening falhar após commit, sem abrir/mover/substituir DB/WAL/SHM vivos.
  `import-postcommit-red.log` comprovou row alterada e conexão em autocommit, mas resultado applied0.
- Correção preserva Storage: callback concluído + erro da API é unknown/applied null/attempted N;
  erros anteriores continuam not_applied/applied0. Mensagem de incerteza não expõe caminho interno.
- Parser aceita separador space/tab/EOL e chaves escalares com aspas. Sem adicionar deserializador YAML.

Revalidação e nova captura/revisão pendentes; nenhuma evidência do alvo v1 aprova este delta.

## Revalidação das correções de revisão

`import-review-green.log`: 100 PASS/1 probe manual ignorado, 2.12 s, features obrigatórias do CI no Mac.
Revisor confirmou 17 paths no target3df5fe646d44cc3face560eb53d1a1b4aad6dc3285b898a91d4e2a50f1ac6939,
manifest a336a488219aec7005d16640fb93475a16b946f0d015158f077606fedab865f5. Dois achados resolvidos
na fonte, nenhum novo no delta. Registro estático em reviews/2026-11-01-markdown-import-safety-static.md.
Linux congelado corresponde aos 17 paths, incluindo ausência de lookup.rs; sem rede/credenciais,
mesmo boundary do export, imagem Rust1.97.1 arm64. Primeira etapa 100 PASS/1 probe manual ignorado,
1.18 s; integrações/full ainda a registrar. Mac pin1.96.0: validação nativa Linux funcional em
compiler diferente é declarada, não é alegada paridade de toolchain ou benchmark entre plataformas.

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

Linux arm64/Rust1.97.1: 100 PASS/1 probe manual ignorado + permission9, routing3, G1 8 (helper1
ignorado mas invocado pelos pais), vault5 PASS; container exit0. Log import-linux.log. Os 17 paths
v2 foram conferidos no input Linux; só baseline documental foi acrescentado depois, validado pelos
checks estáticos/CI no Mac. Não alegar execução Linux do CI inteiro nem toolchain pin1.96.

Mac pin1.96: sensors full PASS timestamp2026-11-01T16:41:04Z duration422 s, sem exclusões; log
import-full.log + receipts canônicos. import-ci-complement-final.log: 13 comandos PASS, incluindo
orçamentos históricos (não são benchmark do candidato), contratos Python/SDK, referência/public
claims, superfícies anunciadas e PDF packaging. Nenhuma falha instável neste full. Hook/pre-commit
com fmt+clippy all-targets/all-features será mantido ao commitar. Sem merge/tag; formal post-gate
exige operador humano/receipt externo (CODE_REVIEW_POLICY linhas111–122), ainda pendente.
