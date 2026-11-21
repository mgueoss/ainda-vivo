# AINDA VIVO v0.0.11 — Colapso

Este documento descreve as regras internas e revela desfechos. O pacote Windows x64 usa o inicializador universal **AINDA-VIVO.exe**. [Release v0.0.11](https://github.com/mgueoss/ainda-vivo/releases/tag/v0.0.11) · [Validação, capturas e distribuição](docs/VALIDACAO-v0.0.11.md)

## Base e escopo

A base é o commit `72d9639`, que integra a revisão do menu à v0.0.10. A expedição segue na Estação 07, com o fusível, as duas leituras da Bomba 02, o rearme, a comporta e a investigação opcional da OS 17. Colapso acrescenta respostas à exposição, refúgios iluminados, recuperação e uma sequência de derrota terminal. O cenário usa a planta e os colisores existentes. A Presença da vistoria e o monitor do Controle seguem os sistemas da versão anterior. [Handoff v0.0.10](AINDA-VIVO-HANDOFF-v0.0.10.md)

O projeto continua em Rust 2024 com Bevy 0.18.1, física cinemática a 60 Hz e distribuição Win32. A instabilidade é um recurso ficcional do operador e dos efeitos da Presença.

## Propriedade de estado

| Camada | Responsabilidade |
|---|---|
| `crates/ainda_vivo_core/src/psychology.rs` | Modelo de um operador com índice de 0 a 100, níveis, exposição, adaptação, recuperação, reserva, temporizador e pulsos motores. `Tuning` concentra os parâmetros. |
| `src/app.rs` e `src/desktop.rs` | A sessão contém o estado psicológico. O passo fixo aplica recuperação, movimento e transições. O controlador cinemático continua responsável pelos limites físicos do corpo. |
| `src/psychology.rs` | Integração com a sessão e o diretor, comandos de diagnóstico e dados de apresentação em `Effects`. |
| `src/psychology/presentation/` e `src/atmosphere/sound.rs` | Silhueta de despedida, sinais visuais e corporais, mistura estéreo, abafamento e efeitos acústicos. |
| `crates/ainda_vivo_core/src/reality.rs` | Verdade da expedição, estado percebido e consequências de Despertar. A captura altera a realidade percebida sem reatribuir o índice psicológico. |
| `src/ui` e `src/preferences.rs` | Acessibilidade, leitura opcional de urgência e preferências locais. |

O estado psicológico pode ser instanciado por operador. Seus dados de apresentação são locais. A lógica recebe a percepção e o contexto de recuperação como entradas e funciona nas duas realidades.

## Instabilidade e recuperação

| Faixa na subida | Estado |
|---|---|
| 0 a 19 | Estável |
| 20 a 39 | Inquietação |
| 40 a 59 | Paranoia |
| 60 a 79 | Desorientação |
| 80 a 99 | Pré-colapso |
| 100 | Colapso |

A descida passa pelas mesmas fronteiras com 3 pontos de histerese. O estado crítico usa o limiar próprio de recuperação abaixo de 85. A configuração inicial atribui 1 a 3 pontos a ruídos, 3 a 5 a anomalias, 5 a 10 a avistamentos, 8 a 15 a observação prolongada, 10 a 19 a alucinações de origem externa e 15 a 24 a contato. Intensidade, tempo entre ocorrências e repetição modulam o ganho. Os limites de acúmulo separam contato de eventos ordinários.

Cada exposição relevante tem uma identidade e origem. O histórico circular registra até 24 contribuições aceitas. Sintomas privados podem aparecer durante a subida e deixam o índice intacto. A escuridão e a passagem do tempo permitem exploração e períodos quietos. Durante a crise, o diretor preserva pausas e pode favorecer sons ou alterações percebidas.

Há cinco postos de pausa sinalizados com luz âmbar, no Acesso, em Bombas, em Tanques, na Elétrica e na Decantação. A luminária de mesa acesa no Arquivo também serve de refúgio. Permanecer em área reconfortante recupera 2 pontos por segundo. Interagir com **E** no posto e ficar parado ativa descanso a 3,2 pontos por segundo. O mesmo posto pode conceder uma reserva fictícia de foco por tentativa. Com **R**, o operador consome a reserva e recupera até 30 pontos de imediato. A recuperação natural começa após 14 segundos de tranquilidade, a 0,2 ponto por segundo, com piso em 12 para esse método.

## Janela crítica e despedida

O colapso inicia uma janela de 60 segundos de simulação. Respiração e pulsação entram primeiro. Ecos privados crescem no segundo quarto, a ambiência fecha gradualmente no terceiro e os sinais expressam urgência no último. O comando de movimento continua disponível. Hesitação e perda breve de corrida usam o controlador normal. Os pulsos motores mais intensos têm intervalo próprio. A pausa congela a janela e a apresentação de despedida.

A crise termina ao descer de 85. A recuperação que cabe dentro do último passo de tempo é aplicada antes de decidir a expiração. Desde 100 pontos, em condições contínuas de abrigo, essa queda leva pouco mais de 7,5 segundos. No descanso ativo, pouco mais de 4,7 segundos. A reserva também pode interromper a crise se estiver disponível e for usada a tempo.

Na expiração, **A despedida** dura cerca de 8,5 segundos. O operador perde força e se ajoelha, a lanterna enfraquece, o áudio se dissolve e a figura surge em posição ancorada. A despedida usa uma silhueta procedural de uso local, com inclinação da cabeça e aceno lento. A gravação escurece e a tela **Registro perdido** oferece uma nova expedição. A sequência encerra a tentativa pelo desfecho psicológico. Despertar continua independente e fica disponível na pausa até começar a despedida.

Um novo início reinicializa os valores, a janela e a reserva. O retorno que pertence à mesma tentativa preserva o estado psicológico. Essa separação mantém a instabilidade compatível com a captura silenciosa e com a incerteza sobre a realidade percebida.

## Acessibilidade e operação

Em **Configurações → Acessibilidade**, ajuste:

- **Intensidade audiovisual**, em passos de 25% de 0 a 100%.
- **Câmera nas crises**, com redução do movimento adicional.
- **Indicador de estabilidade**, oculto por padrão. Quando ligado, mostra a faixa atual e um rótulo graduado de urgência na crise.

As opções de apresentação usam as mesmas regras de exposição e recuperação. O tempo exato e as origens dos estímulos aparecem nas ferramentas de diagnóstico. O painel de acessibilidade contém um aviso breve de pânico, alterações sensoriais e colapso sobrenatural. Os postos também orientam o uso de descanso e reserva.

## Desenvolvimento e diagnóstico

Execute o jogo com `--dev-psychology` para habilitar:

| Tecla | Comando |
|---|---|
| F9 | Consultar valor, estado, temporizador e fontes recentes. |
| F10 | Aumentar o índice em 10 pontos. |
| Shift + F10 | Reduzir o índice em 10 pontos. |
| F11 | Percorrer os níveis psicológicos. |
| F12 | Solicitar um acontecimento psicológico privado. |
| Shift + F12 | Testar diretamente a despedida. |

`--dev-reality` continua a habilitar F6 para captura de teste, F7 para solicitar ilusão e F8 para consultar o estado de realidade. Os dois modos de desenvolvimento podem ser combinados para verificar a mesma condição psicológica em realidades diferentes.

```powershell
cargo fmt --all -- --check
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
cargo test --locked --offline -p ainda-vivo -p ainda_vivo_core
cargo test --locked --offline -p ainda-vivo-launcher
cargo build --locked --offline -p ainda-vivo
```

As rotas nativas e o empacotamento são descritos no relatório versionado. A aferição definitiva do medo, conforto e diversão cabe ao diretor e aos testers durante partidas livres. [Relatório v0.0.11](docs/VALIDACAO-v0.0.11.md) · [Distribuição](docs/DISTRIBUICAO.md)

## Referências de design aplicadas

- **Amnesia: The Dark Descent**. A experiência com escuridão e feedback descrita por Thomas Grip orientou sinais corporais e a ocultação inicial do índice. Aqui, a pressão vem de encontros percebidos. [Relato de Thomas Grip](https://www.gamedeveloper.com/design/game-design-deep-dive-i-amnesia-i-s-sanity-meter-)
- **Amnesia: Rebirth**. O pós-mortem relata leituras conflitantes sobre as consequências do colapso. Comentários de jogadores citam pressão contínua, clarões e interrupções. A janela de recuperação usa deslocamento normal, sinais graduais e uma consequência legível no fim. [Pós-mortem e discussões](https://frictionalgames.com/2021-03-amnesia-rebirth-five-months-later-and-soma-sells-1m-copies/) · [Ajuste oficial dos clarões](https://frictionalgames.com/2020-10-amnesia-rebirth-updated-to-1-04/)
- **Eternal Darkness** e **SOMA**. A fragilidade da percepção serve para pôr em dúvida o que se vê na Estação. A captura e a instabilidade são independentes para sustentar a incerteza sobre a experiência do operador. [Entrevista de Denis Dyack](https://www.shacknews.com/article/47154/denis-dyack-interview) · [Frictional sobre SOMA](https://frictionalgames.com/2015-09-frictional-games-presents-soma-available-now/)
- **Darkest Dungeon**. O relato de Red Hook sobre aflições e histórico individual orientou reações breves e um histórico por operador. [Relato de Chris Bourassa e Tyler Sigman](https://www.gamedeveloper.com/design/game-design-deep-dive-i-darkest-dungeon-s-i-affliction-system)
- **Darkwood**. Preparação e antecipação orientaram os refúgios e a alternância entre exposição e silêncio. A relação não linear entre medo e diversão observada em uma atração de horror inspirou a variação de intensidade. Os limiares do jogo são escolhas ficcionais de balanceamento. [Acid Wizard Studio](https://www.darkwoodgame.com/about/) · [Andersen et al.](https://pure.au.dk/ws/files/219262042/0956797620972116.pdf)
