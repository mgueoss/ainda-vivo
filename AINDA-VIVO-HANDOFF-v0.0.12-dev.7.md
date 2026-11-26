# AINDA VIVO v0.0.12-dev.7 — Guerra Psicológica

Esta é uma build experimental local da Estação 07. A base é a dev.6 auditada em 25 e 26 de novembro de 2026, commit `00f69f0b6f5ad43027b8e0ce6e252891a4b27c45`. A edição comum abre com a Presença 3D. O pacote de avaliação também inclui o Dev Lab.

## Onde continuar

| Arquivo | Responsabilidade |
|---|---|
| `src/presence.rs` e `src/presence/rig.rs` | Plugin de campanha, carregamento do catálogo e reprodução dos clipes. Os recursos da criatura são embutidos. |
| `src/presence/observer.rs` | Integra a campanha com o cérebro, colisores, percepção, relé, diretor, captura e sinais audiovisuais. Gera as contagens de oportunidade. |
| `src/presence/observer/{brain,cognition,tactics,strategy}.rs` | Visão, audição, crenças e histórico. Decisão local, planejamento limitado e avaliação. |
| `src/presence/observer/{nav,hunt}.rs` | Rotas com volumes por postura, portas e execução física. O estado `Narrow` termina o sprint antes da retomada em pé. |
| `src/atmosphere/{director,sound/presence_audio}.rs` | Janelas, resolução de pedidos do ator e apresentação sonora. Retorno de entrega com escopo por tentativa e revisão. |
| `src/lab.rs`, `src/lab/panel.rs` e `src/presence/observer/{test_lab,hunt_lab}.rs` | Interface e recursos de ensaio. Dependem de `dev-lab` e da ativação explícita. |
| `src/smoke/campaign.rs` | Roteiro de campanha física com movimento, raio de interação e desfecho reais. |
| `src/preferences.rs` e `src/diagnostics.rs` | Gravação assíncrona com substituição, novas tentativas e retenção de diagnóstico de sessão. |
| `crates/ainda_vivo_launcher` e `scripts/distribution/package.ps1` | Inicializador independente e ZIP. Aceitam `-dev.N` e o atalho explícito de Lab. |

O planejador jamais recebe a ação futura do operador. Insira novas estratégias em `Mind` usando as pistas de `Frame` que passam pelos sensores. Use o motor já existente para destinos e preserve a exclusividade entre caminhada e caça. Trate falha de um pedido perceptivo como decisão de apresentação. Uma inferência errada deve ser possível. Não transforme a edição de diagnóstico em requisito para renderizar a criatura.

## Compilação e ensaios

```powershell
cargo fmt --all -- --check
cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
cargo test --locked --offline --workspace --features dev-lab -- --nocapture
cargo build --locked --offline
cargo build --locked --offline --features dev-lab
cargo rustc -p ainda-vivo-launcher --profile launcher --locked --offline -- -C target-feature=+crt-static
```

Em Windows, use o executável em `target\debug` ou em `$env:CARGO_TARGET_DIR\debug` quando o destino estiver configurado. Para comparação entre builds, copie cada executável para um nome distinto logo após compilar. Confira os relatórios e o código de saída.

```powershell
.\ainda-vivo.exe --smoke-campaign --campaign-profile explorador --campaign-hz 120 --seed 42 --trace-presence --smoke-report campanha.txt
.\ainda-vivo.exe --smoke-test --check-audio --capture-dir capturas-legado --smoke-report legado.txt
.\ainda-vivo.exe --smoke-echoes --check-audio --smoke-report os17.txt
.\ainda-vivo.exe --smoke-hunt --capture-dir capturas-cacada --smoke-report cacada.txt
.\ainda-vivo.exe --check-strategy --smoke-report estrategias.txt
.\ainda-vivo.exe --dev-lab --ai-seed 42017
```

O roteiro legado é intencionalmente vinculado à silhueta. A opção antiga `--smoke-test --presence-physical` é redirecionada para a campanha física. Para reconstruir o ZIP, passe o inicializador compilado e o executável com Dev Lab a `scripts/presenca/empacotar-dev.ps1`. Use os avisos de licenças da dev.6, que já cobrem o conjunto de dependências compartilhado pela dev.7. Consulte o [guia de distribuição](docs/DISTRIBUICAO.md).

## Ponto de retomada para a direção

Use o relatório de validação para distinguir cenas verificadas de critérios artísticos pendentes. O foco recomendado é equilibrar encontros com visibilidade mútua e espaço de fuga, em sessões longas com pessoas, antes de promover os perfis mais rápidos. Repita e compare a diversidade de planos, a interpretação dos sons em salas distintas e o recuo em torno dos portais da Estação. Registre ocorrência, ausência ou abandono de perseguições e preserve as rotas automatizadas como proteção contra regressão.

[Estratégias e arquitetura](docs/GUERRA-PSICOLOGICA-v0.0.12-dev.7.md) · [Relatório de testes](docs/VALIDACAO-v0.0.12-dev.7.md) · [Relatório consolidado](AINDA-VIVO-RELATORIO-GUERRA-PSICOLOGICA.txt)
