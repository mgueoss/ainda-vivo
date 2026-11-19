# AINDA VIVO — handoff v0.0.9

**Falsa Realidade · Windows x64 · Estação de Bombeamento 07**

Documento para manutenção, com spoilers. O usuário tem a direção criativa. A v0.0.9 parte do jogo v0.0.8 (tag no commit `24f782f`) e da distribuição universal (commit `625c45e`). A [filosofia permanente](AGENTS.md), as [referências de design](docs/REFERENCIAS-FALSA-REALIDADE.md) e o [changelog](CHANGELOG.md) orientam a próxima etapa. A versão publicada e o fonte são identificáveis pela tag `v0.0.9`. [Release v0.0.9](https://github.com/mgueoss/ainda-vivo/releases/tag/v0.0.9)

## Jogo e progresso

O operador atravessa a oficina, pega o fusível, registra a pressão de repouso da Bomba 02, rearma o quadro geral, volta ao manômetro e atravessa a Decantação. O mapa tem 15 setores e 22 portais. A caixa cinemática do personagem roda a 60 Hz. Os módulos de estação, iluminação, câmera fixa, lanterna, VHS, conforto, som espacial e preferências continuam em uso.

A instrução do posto fica ao lado do quadro geral. Ela sugere iluminar o vão ou recuar caso os passos retornem. A identificação de Borges está na nave, perto da passagem para a Decantação. A assinatura, as ocorrências e os horários continuam disponíveis no Arquivo e no Controle. A câmera 02 mantém sua composição opcional. O novo visor no painel do Controle inclui o horário da linha. Nenhuma dessas leituras opcionais é requisito para abrir a comporta.

| Módulo | Responsabilidade |
|---|---|
| `crates/ainda_vivo_core/src/story.rs` | Fases e interações da vistoria em `Progress`. |
| `crates/ainda_vivo_core/src/reality.rs` | Estado autoritativo, observador, estado privado e desfechos. A máquina `Approach` aceita tempo e condições já aferidas. |
| `src/app.rs`, `src/desktop.rs` | Coordenação do Bevy e da sessão. Entrada, corpo, câmera, relógio fixo, portaria e reposição no posto. |
| `src/reality.rs`, `src/scene/presence.rs` | Vestígios avistados, condições espaciais da aproximação, detecção da exposição e ferramentas de desenvolvimento. |
| `src/scene/illusion.rs` | Catálogo de adereços perceptivos e material do leitor da portaria. |
| `src/atmosphere/director.rs` | Decisões de ritmo, janela, histórico e seed. |
| `src/atmosphere/sound.rs`, `src/atmosphere/episode.rs` | Áudio espacial, variantes de passos, aviso local e estados das sequências autorais. |
| `src/ui.rs` | Confirmação de Despertar, menu e HUD baseado no que o operador percebe. |
| `src/smoke.rs`, `src/smoke/continuity.rs` | Rotas gráficas de regressão e de continuidade. |

## Autoridade e percepção

Em `Expedition`, `truth` guarda o progresso autoritativo e `observer` guarda o participante. `ObserverId` é estável por tentativa. O operador solo recebe `ObserverId::LOCAL`. `Expedition::view()` resolve a progressão usada por interação, objetivos, câmera fixa e colisão da comporta local. Em realidade verdadeira o visor lê o progresso real. Na captura, a percepção recebe uma cópia de `Progress` e passa a evoluir separadamente. O mapa, o relógio da sessão, a entidade ECS da lanterna e o corpo cinemático continuam sendo os mesmos.

| Estado | Regra |
|---|---|
| `Reality::Real` | As ações de missão avançam o mundo. As quatro anomalias visuais da primeira camada continuam possíveis. |
| `Reality::False` | As ações avançam o registro do observador. A leitura privada pode abrir a comporta no seu visor sem alterar o mundo verdadeiro. A influência adicional do diretor começa após 4 s de jogo. |
| `Reality::Eliminated` | O personagem encerrou a participação ao confirmar Despertar enquanto estava vivo. A tela é de registro interrompido. |

O `Outcome` é independente. `Escaped` conclui a expedição ao cruzar a saída com a vistoria real em dia. `PerceivedExit` mostra a baixa da portaria e produz, quatro segundos depois, uma contradição no visor e um som familiar no corredor. O operador pode continuar se movendo ou abrir a pausa. `Interrupted` finaliza a tentativa pela decisão de Despertar em realidade verdadeira.

A solução é local à única câmera desta versão. Em LAN, mova o `Progress` autoritativo para a estação do host e o `Observer` privado para cada participante. O registro de participação é a futura fronteira para ocultar o corpo capturado das outras pessoas sem retirar seu cliente da experiência. O filtro da comporta, o visor, as ilusões e o áudio precisarão ser resolvidos por observador. Aproveite os mesmos assets e alinhamentos espaciais.

## Presença e captura

As dez posições autorais de aparição reutilizam as linhas de visão auditadas. As aparições distantes exigem enquadramento e visibilidade. Ao revisitar um setor em movimento, o diretor pode preferir uma posição elegível à frente. No vão oeste, a figura espera e pode avançar até 0,83 m dentro do enquadramento original. Uma luz de preenchimento reaproveitada destaca seu contorno contra o concreto. A entidade se retira quando o operador chega perto e sua intenção persiste por meio do vestígio.

A captura natural é elegível após o rearme verdadeiro. No vão oeste da nave ou no retorno da porta da Elétrica, uma aparição enquadrada por 0,46 s deixa um vestígio com 24 s de validade. Aproximar-se a 4,4 m ou menos, com acesso visual livre para o ponto, dispara um aviso sonoro ambíguo. Durante a janela de 6 s, ficar no chão, abaixo de 1,55 m/s e com o olhar afastado acumula exposição. O acúmulo precisa ser contínuo por 2,15 s e a captura respeita pelo menos 2,4 s desde o aviso. Virar-se para a origem corta a exposição. A lanterna ligada amplia o ângulo que protege. Sair da zona ou perder o acesso físico encerra a sequência. A figura também pode envelhecer e desaparecer sem produzir contato.

A transição altera apenas a autoridade das ações seguintes. O frame de captura conserva o movimento, a orientação, a lanterna, o HUD habitual e o relógio. Nenhum sinal do estado real entra no diagnóstico F3. A rotina de aviso já está em andamento antes da transição.

## Despertar

`Esc` abre a pausa. Despertar apresenta a mesma confirmação em ambas as realidades. Voltar ou pressionar Esc fecha a confirmação. Enter fica inativo enquanto a decisão está pendente.

Na falsa realidade, a ação recupera o mesmo operador no marcador `RETURN_POST` da Elétrica, fixado pelo rearme. O relógio não é rebobinado. O visor volta a ler o mundo e conserva o progresso real anterior à captura. Se apenas a leitura privada abriu a comporta, a segunda leitura continua faltando no mundo. Se a segunda leitura já foi registrada de verdade, a comporta permanece aberta. A revisão da percepção expira os sons privados, restaura adereços, limpa o vestígio e reserva um intervalo sem novas janelas. O retorno existe uma vez por tentativa. Depois dele, a captura fica indisponível até um reinício completo.

Na realidade verdadeira, a decisão confirmada encerra a tentativa. A apresentação diz apenas que o registro foi interrompido. Reiniciar cria outro observador e restaura a fase do fusível, sons, gravação, objetivo, ilusões e relógio.

## Diretor, catálogo e áudio

O contexto usa horário ativo, fase percebida, máscara de setores visitados, as quatro visitas mais recentes, rotação da câmera e velocidade horizontal. Guarda os seis últimos eventos, último horário da Presença, intensidade recente com decaimento, e a força da distorção local. A amostragem de contexto do diretor ocorre a cada 0,34 s. As janelas têm prioridade, expiração, alternância de categoria e proteção para sons autorais e avisos de contato. Inquietação e antecipação se somam a tranquilidade, suspeita, tensão e alívio. Janelas podem terminar sem evento. Nenhuma ordem de modos é obrigatória.

O catálogo tem ficha, indicador de ramal, relógio e etiqueta de turno. Cada entidade usa uma pose e um material base e uma variação. Para trocar, o operador precisa ter visto o item de perto com raio desobstruído, virar o olhar e encontrar uma janela ativa. Ficha e relógio têm uma segunda possibilidade depois de alteração, tempo, nova inspeção e revisita na falsa realidade. As trocas são audiovisuais. A comporta só consulta a segunda leitura da vistoria local. A tela da portaria usa três materiais na mesma malha. O mundo tem uma única estação renderizada.

Os passos ocasionais misturam sapato abafado e vibração de metal. Uma sequência atrás do operador reage à velocidade e para de emitir quando o operador para. O aviso espacial do contato toca perto do vão. A portaria repete um som conhecido num ponto distante da oficina. Os emissores compartilham os canais estéreo, atenuação, oclusão e volume de ambiência. A pausa congela a janela e pausa os sinks. Os sinais visuais funcionam com VHS desligado.

## Depuração e manutenção

- Use `--dev-reality` para habilitar F6, F7 e F8. F6 pede captura, condicionada a um rearme real e tentativa elegível. F7 pede uma oportunidade de ilusão válida. F8 registra realidade, fase real e percebida, desfecho, seed, modo e eventos recentes no log. O HUD normal continua igual.
- `--seed N` fixa o gerador do diretor e dos incidentes para reproduzir uma tentativa com a mesma entrada. Um replay bit a bit de toda a simulação entre máquinas não é um recurso desta versão.
- Use `--smoke-test --check-audio` para o percurso de regressão. Use `--smoke-reality --seed 20261118 --check-audio` para a rota de continuidade. Ambos aceitam `--smoke-report ARQUIVO` e `--capture-dir PASTA`. As pastas de captura devem existir. Os testes rodam com tempo virtual de 60 Hz, janela e hardware real. A rota específica posiciona o operador para as interações focadas, mas aproximação, recuo e travessia interagem com a física real.
- Dependências do jogo estão travadas no `Cargo.lock`. O pacote Windows usa o launcher universal v1.0.0, a rotina de preparo e os avisos de terceiros. [Distribuição](docs/DISTRIBUICAO.md) · [Testes e capturas](docs/VALIDACAO-v0.0.9.md)

## Limitações e próximos passos

Rustfmt, Clippy com avisos fatais, compilação offline e 41 testes passaram. O smoke gráfico e sonoro confirmou 12.260 quadros na rota regular e 2.696 na rota de continuidade. Uma segunda seed passou em outros 2.696 quadros com áudio mudo. O ZIP foi extraído e aberto pelo inicializador universal. Os SHA-256 e as capturas reais estão no [relatório de validação](docs/VALIDACAO-v0.0.9.md).

A captura natural começa em duas posições autorais com acesso geométrico e deslocamento curto. O áudio é procedural. As pistas de manutenção usam objetos e registros escritos. O mapa é solo. O teste de continuidade observa os desfechos e a estabilidade dos estados. Frequência espontânea de captura, resposta emocional, conforto subjetivo e desempenho em partidas livres precisam de sessões humanas separadas.

A próxima avaliação deve observar se jogadores identificam o aviso antes de descobrir as regras, se entendem o peso de Despertar e se o retorno é curto o bastante. Só depois ajuste pesos de aparição, contraste local ou economia do checkpoint. Em LAN, definir autoridade e fronteira de apresentações privadas antes de imitações entre jogadores.
