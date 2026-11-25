# AINDA VIVO v0.0.12-dev.6 · Inteligência Hostil

Base funcional `27d5e1e`, v0.0.12-dev.5. A edição experimental usa Rust, Bevy 0.18.1 e a física cinemática existente. O pacote abre com a Presença física. **F5** permite comparar a representação anterior e **Ctrl+I** entra no Dev Lab.

## Continuidade técnica

- O cérebro vive em `src/lab/observer`. Leia `brain.rs`, `cognition.rs` e `tactics.rs` antes de ajustar decisões. `hunt.rs` executa planos sob a lógica de caçada, com a grade em `nav.rs`.
- Pistas da visão e audição vencem com o tempo. A adaptação é local à tentativa e usa travessias observadas e reveses. O reinício e a revisão perceptiva reinicializam o cérebro da campanha.
- A posição verdadeira participa do sensoriamento, das verificações físicas e dos efeitos da sessão. O planejador depende das pistas registradas. `RelayClick` é a ponte sonora de uma ação física. `Request` em `presence_audio.rs` só conduz apresentação.
- `Director` fornece oportunidades e controla sobreposição e recuperação. `choreography.rs` escolhe uma intenção e muda o andamento conforme reação ou contexto. As ilusões não modificam a progressão real nem o grafo.
- O GLB, os clipes, o mapa e os resultados narrativos de contato continuam sendo os da base. A classe de contato depende da sessão e da elegibilidade existentes.

## Reproduzir

```powershell
cargo fmt --all --check
cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings
cargo test --locked --offline --profile dev --workspace --features dev-lab
cargo build --locked --offline -p ainda-vivo --features dev-lab
.\target\debug\ainda-vivo.exe --check-cognition --smoke-report matriz.txt
.\target\debug\ainda-vivo.exe --smoke-inteligencia --seed 20261118 --smoke-report lab.txt --capture-dir capturas
.\target\debug\ainda-vivo.exe --dev-lab --presence-physical --ai-seed 42017
```

Em uma máquina com `CARGO_TARGET_DIR`, use o executável em `$env:CARGO_TARGET_DIR\debug`. Para distribuição, reutilize `scripts/presenca/empacotar-dev.ps1` com os avisos de licenças da build anterior.

## Próxima avaliação

Observe rotas reais variando entre andar, correr, interromper visão em esquinas e voltar a um setor conhecido. Compare uma recuperação que funciona com uma perseguição em que a Presença desiste. Registre cenas em que as prioridades parecem transparentes ou injustas com o diagnóstico exportado. [Arquitetura e regras](docs/INTELIGENCIA-HOSTIL-v0.0.12-dev.6.md) · [Testes](docs/VALIDACAO-v0.0.12-dev.6.md) · [Referências](docs/REFERENCIAS-IA-v0.0.12-dev.6.md)
