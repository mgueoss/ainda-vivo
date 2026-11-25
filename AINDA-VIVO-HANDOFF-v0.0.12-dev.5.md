# AINDA VIVO v0.0.12-dev.5 · Ecos da Caçada

Continuação da dev.4 a partir de `2a1fbbe`. A etapa implementa a identidade sonora da Presença e as ilusões de aproximação sobre Rust, Bevy 0.18.1, física própria e o banco procedural de áudio existente. A edição experimental usa `--features dev-lab`. O GLB, o rig, a navegação e os resultados de contato continuam nas camadas já usadas pela dev.4.

## Onde dar manutenção

| Arquivo | Responsabilidade |
|---|---|
| `src/atmosphere/sound/presence_bank.rs` | Síntese, versões seca/refletida, variações por superfície e teste WAV |
| `src/atmosphere/sound/presence_audio.rs` | Sinais da animação, seleção de amostras, sessões, oclusão, saturação e ilusões |
| `src/atmosphere/sound/spatial.rs` | Decodificador estéreo com filtro e transição de ganho usado pelo jogo |
| `src/lab/library.rs` e `src/lab/observer/hunt.rs` | Contatos nos clipes e sinais das fases de perseguição |
| `src/lab/sound_lab.rs`, `panel.rs`, `sound_qa.rs` | Inspeção sonora, comandos e roteiro automatizado Windows |

O diretor controla janelas e frequência. A IA recebe os estímulos lógicos que já existiam. `Request` de áudio é somente apresentação. As três origens diferem em contexto e diagnóstico e usam o mesmo banco. A falsa realidade é lida do observador da expedição. A troca de tentativa ou revisão perceptiva reinicia as fontes da campanha. O áudio do Dev Lab permanece isolado da missão.

## Reproduzir

```powershell
cargo fmt --all --check
cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings
cargo test --profile dev --locked --offline --workspace --features dev-lab
cargo build --locked --offline -p ainda-vivo --features dev-lab
.\target\debug\ainda-vivo.exe --smoke-sonoro --seed 20261118 --smoke-report resultado-som.txt
.\target\debug\ainda-vivo.exe --dev-lab --presence-physical
```

Se o ambiente usar `CARGO_TARGET_DIR`, leia o binário de lá. O empacotamento experimental usa `scripts/presenca/empacotar-dev.ps1` e as licenças da build anterior.

## Continuidade

O ajuste seguinte é ouvir lado a lado mãos, pés e voz sob diferentes obstáculos e níveis de ambiência, avaliar o contraste entre caça física e invisível e medir o ritmo em rotas jogadas por pessoas. A extensão natural da modelagem acústica seria verificar quais portas influenciam a leitura da sala e corrigir pontos específicos. As regras de duração, volume, alcance e elegibilidade estão registradas na documentação do áudio. [Guia de teste](docs/DEV-LAB.md) · [Áudio](docs/AUDIO-PRESENCA-v0.0.12-dev.5.md) · [Validação](docs/VALIDACAO-v0.0.12-dev.5.md)
