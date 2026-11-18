# Arquitetura da v0.0.1

Este capítulo registra o protótipo original. Para os módulos e fluxos atuais, consulte a [documentação de continuidade da v0.0.8](CONTINUIDADE-v0.0.8.md).

## Limites entre módulos

| Local | Responsabilidade |
|---|---|
| `crates/ainda_vivo_core` | Matemática, colisões, movimento e regras do objetivo; sem janela, renderização, sistema operacional ou rede |
| `src/app.rs` | Composição, sessão e preferências de conforto |
| `src/desktop.rs` | Teclado/mouse, física fixa, câmera interpolada e captura do cursor |
| `src/scene.rs` | Mapa e materiais procedurais; mesma definição gera visual e colisão |
| `src/interaction.rs` | Mira, alcance e oclusão de interação |
| `src/ui.rs` | Menu, pausa e interface de gravação |
| `src/diagnostics.rs` | FPS, tempo de quadro, posição e contato com chão |
| `src/smoke.rs` | Verificação limitada de inicialização e integração com renderização |

O mundo é medido em metros, Y para cima, frente inicial em −Z. O jogador inicia em `(0, 0.01, 8)`. A sala do fusível fica em `(-5, 4)` no plano X/Z; o registro em `(5.2, -4)`; o quadro em `(1.42, -8.6)`; a saída em `z < -10.7`.

## Movimento e colisão

O adaptador de entrada escreve `PlayerCommand`, não move a câmera diretamente. A cada passo fixo de 1/60 s, `PlayerBody` aplica velocidade e gravidade e faz uma varredura do volume contra as caixas estáticas. Usa expansão de Minkowski, tempo de impacto e até oito planos de contato, com margem de 1 mm. A câmera interpola os dois estados físicos e suaviza apenas a altura ao agachar. Mouse não é multiplicado pelo tempo de quadro. Movimento diagonal é normalizado.

Pausa interrompe o avanço do corpo, da missão e do relógio. Perda de foco limpa o comando e exige continuar a partida. A porta de saída tem visual e colisão desativados pela mesma condição de progresso. Reiniciar restaura jogador, missão, objetos e lanterna.

Interação usa raio até 2,4 m e verifica primeiro as paredes e móveis que podem bloquear a visão. Os alvos interativos possuem uma margem pequena para serem fáceis de selecionar; não atravessam paredes.

## Android, posteriormente

Criar um adaptador de toque que produza `PlayerCommand`, com UI e sensibilidade próprias. Preservar o núcleo de regras. Acrescentar pausa por ciclo de vida, configuração de renderização móvel, controle de orçamento térmico e empacotamento Android. A separação atual facilita essa adaptação; ela não significa que o projeto já compila ou foi validado no Android.

## LAN de 2–4 jogadores, posteriormente

Antes de escolher transporte, extrair a sessão para uma simulação com identificadores estáveis de jogadores, objetos e ticks. O host será autoridade sobre posição, inventário, portas e criatura. Clientes enviarão comandos numerados; o host validará velocidade, alcance e oclusão. Implementar snapshots, interpolação de jogadores remotos e depois predição/reconciliação local.

`PlayerCommand` é uma fronteira inicial reutilizável, ainda sem serialização ou protocolo. Não presumir determinismo entre plataformas por usar passo fixo: a simulação usa floats e deve reconciliar pelo host. A LAN não deve abrir portas na internet automaticamente.

Ilusões devem ficar no estado de apresentação **por jogador**, separadas do mundo autoritativo. Morte silenciosa deve ser um estado de jogabilidade com regras explícitas, sem depender de efeitos de câmera. O modo solo narrativo poderá consumir a mesma simulação sem transporte de rede.

## Decisões de escopo

Não há framework de rede, servidor HTTP, instalador, pipeline de assets nem abstrações vazias para funcionalidades futuras. A primeira versão valida os controles e o corredor. Quando o cenário passar a exigir rampas ou física dinâmica, avaliar um controlador cinemático sobre Avian/Rapier compatível com a versão de Bevy escolhida; manter testes de comportamento como critério de migração.
