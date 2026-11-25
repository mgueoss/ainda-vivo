# Changelog

## v0.0.12-dev.6 — Inteligência Hostil

- A Presença atualiza aproximações laterais, procura pontos alcançáveis junto a equipamentos e confirma contato com a posição física do operador. Abaixa o ritmo ao perder contato, investiga o último indício e pode retomar a mesma caçada.
- Visão central e periférica, audição aproximada, memória com confiança e idade, planos de busca e pesos de rotas observadas mudam as escolhas entre observar, investigar, intimidar e interceptar.
- O relé da Elétrica pode produzir um estalo atrasado. O diretor reserva espaço para caçadas e escolhe sequências ilusórias conforme pressão, setor e reações observáveis.
- O Laboratório Cognitivo adiciona bots, caixa móvel, avanço de um passo, seeds, memória, matriz de testes e exportação. [Guia](docs/DEV-LAB.md) · [Arquitetura](docs/INTELIGENCIA-HOSTIL-v0.0.12-dev.6.md) · [Handoff](AINDA-VIVO-HANDOFF-v0.0.12-dev.6.md)

## v0.0.12-dev.5 — Ecos da Caçada

- A Presença usa uma biblioteca original com passadas, apoios, atrito, respiração e voz sintetizada. Os marcadores de animação acionam os sons físicos de caminhada e corrida quadrúpede.
- As fontes ganham direção, atenuação, abafamento por obstáculos e reflexões curtas. O diretor pode criar aproximações sonoras e caçadas invisíveis em setores vizinhos com a mesma biblioteca.
- A aba **Som** do Dev Lab reúne amostras, variações, teste de porta, caminhada, caçada física e sequências ilusórias. [Guia](docs/DEV-LAB.md) · [Áudio da Presença](docs/AUDIO-PRESENCA-v0.0.12-dev.5.md) · [Handoff](AINDA-VIVO-HANDOFF-v0.0.12-dev.5.md)

## v0.0.12-dev.4 — A Caçada

- A Presença física pode preparar uma arrancada ou desistir da ameaça. A corrida usa as animações quadrúpedes existentes, adapta a velocidade às curvas e abandona rastros perdidos.
- O contato pode ser ilusório, intimidar, desorientar, capturar silenciosamente ou terminar em retirada. A captura respeita a elegibilidade e o posto de Despertar existentes.
- A aba **Caçada** do Dev Lab testa corrida livre, porta, ameaça, interrupção e os cinco resultados de contato em uma sessão isolada. [Guia](docs/DEV-LAB.md) · [Handoff](AINDA-VIVO-HANDOFF-v0.0.12-dev.4.md)

## v0.0.12-dev.3 — O Observador

- Presença experimental com visão obstruída por paredes e portas, audição aproximada, memória de curta duração e navegação física pelos setores da Estação.
- O diretor oferece janelas para manifestações. A criatura escolhe seus comportamentos com base nos estímulos locais e usa as ações do rig da dev.2.
- Aba **IA** no Dev Lab com porta de ensaio, ruído por alvo, controle manual, rotas, visão e memória. `--presence-physical` seleciona a representação física na campanha experimental. [Guia](docs/DEV-LAB.md) · [Handoff](AINDA-VIVO-HANDOFF-v0.0.12-dev.3.md)

## v0.0.12-dev.2 — Movimentos Antinaturais

- Ações da Presença no mesmo rig e na mesma malha da dev.1. Incluem vigília, passadas humanoides, três rupturas, dois ritmos de corrida quadrúpede, frenagem, retomada da postura em pé e uma despedida lenta.
- Reprodução Bevy por estados, com mistura de poses, apoios e marcadores para sincronização futura. O Lab permite inspecionar a linha do tempo, comparar clipes e ajustar reprodução e deslocamento separadamente.
- Pista com curvas e aproximação encenada em primeira pessoa no Dev Lab. O controle da criatura nessa cena é independente da campanha. [Dev Lab](docs/DEV-LAB.md) · [Handoff](AINDA-VIVO-HANDOFF-v0.0.12-dev.2.md)

## v0.0.12-dev.1 — A Presença e Dev Lab

- Primeira malha original da Presença, esculpida em Blender. O arquivo editável inclui rig, pesos, texturas PBR e poses de inspeção, entre elas o apoio quadrúpede e a despedida.
- Sala interna com catálogo, comparação de modelos, colisores, esqueleto, wireframe, câmeras, cinco luzes predefinidas e player de clipes.
- Acesso de desenvolvimento com `Ctrl+I`. A sessão da Estação é suspensa durante a inspeção. A prévia passiva na campanha tem um parâmetro próprio.
- Build experimental da Etapa 1. [Dev Lab e atalhos](docs/DEV-LAB.md) · [Handoff](AINDA-VIVO-HANDOFF-v0.0.12-dev.1.md)

## v0.0.11 — Colapso

- Seis níveis psicológicos com adaptação a estímulos repetidos, histórico por operador e uma janela crítica recuperável.
- Postos de pausa, refúgio da luminária do Arquivo e uma reserva fictícia de foco por tentativa.
- Respiração, pulso, alterações sonoras e a sequência **A despedida**. Configurações de intensidade audiovisual, movimento da câmera e indicador opcional.
- Integração com a captura silenciosa, Despertar e o ritmo da Presença. [Regras e comandos de diagnóstico](AINDA-VIVO-HANDOFF-v0.0.11.md) · [Validação e distribuição](docs/VALIDACAO-v0.0.11.md)

## v0.0.10 — Ecos do Desaparecimento

- Abertura reformulada com uma arte original da Estação 07, Configurações e Controles em painéis próprios e navegação por mouse ou teclado.
- Pausa, retomada e confirmação de Despertar usam a mesma identidade visual. A revisão do menu integra a v0.0.10. [Detalhes](docs/MENU-v0.0.10.md)
- A estação registra por onde o operador passou, o que consultou, quais objetos observou e o ritmo dos passos. Uma alteração visual precisa de uma referência que ele realmente tenha visto.
- A segunda via da OS 17 liga o Arquivo, a Elétrica e o Controle. Passos fora de quadro, a ficha deslocada e um segundo quadro da Câmera 02 deixam pistas sobre Borges. A vistoria da Bomba 02 independe dessa investigação.
- A luminária do Arquivo oferece uma pausa com transições suaves de luz e de ambiência. A Presença pode voltar à Triagem em rotas que já passaram pelo local.
- Despertar vivo encerra o registro. Depois de rearmar a Elétrica, o jogador pode iniciar uma única retomada curta no posto com o progresso real.
- A leitura completa da OS 17 altera o registro final. A baixa da portaria tem um período de alívio antes da contradição.
- Novos testes de memória e uma rota Windows para validar a cadeia de acontecimentos, os três passos e a retomada. O pacote usa o [inicializador universal](docs/DISTRIBUICAO.md).

## v0.0.9 — Falsa Realidade

- A Presença pode capturar silenciosamente após uma aproximação perigosa. A vistoria continua na perspectiva do operador.
- Despertar, na pausa, é uma decisão confirmada. A história pode chegar a uma saída verdadeira, a uma baixa contraditória ou a um registro interrompido.
- A nave e o Controle ganharam sinais de turno, um relógio sujeito a alterações e registros sobre o posto de retorno. Passos distantes reagem ao deslocamento.
- O diretor usa memória curta de setores e eventos, intensidade recente e seis modos de ritmo. Seeds reproduzíveis e ferramentas de inspeção ficam disponíveis para desenvolvimento.
- O protocolo de distribuição continua usando o inicializador universal e o ZIP portátil Windows x64.

## Distribuição Windows x64

- Os oito pré-lançamentos históricos usam o mesmo inicializador nativo, com verificação de dependências e registro de diagnóstico.
- Os pacotes de jogador reutilizam os executáveis originais. Cada ZIP tem SHA-256 e contém os arquivos necessários para jogar.
- A recuperação e a publicação são documentadas em [Distribuição portátil](docs/DISTRIBUICAO.md).

## v0.0.8 — Além da Escuridão

- Diretor local de terror com tranquilidade, suspeita, tensão e alívio. Considera a fase da vistoria, o setor, revisitas, velocidade, olhar e eventos recentes. Algumas janelas de expectativa passam em silêncio.
- A Presença alterna imobilidade, observação, espiada e travessia. Pode voltar a um setor revisitado e se retirar quando sai do campo de visão após ser percebida.
- Duas ilusões visuais experimentais alteram elementos de cenário depois que o jogador os observa e olha para outro lado. Passos fora do enquadramento complementam as ilusões.
- Sons incidentais em três variantes por família. Revisão das transições de passos, goteiras, ventilação, ressonância da bomba e arrasto de metal.
- Livro de ponto no Controle, horários contraditórios no Arquivo e novos textos associados à câmera da nave e às leituras da Bomba 02.
- Lanterna com núcleo um pouco mais concentrado, desgaste procedural nas paredes e VHS mais discreto. O balanço opcional tem amplitude máxima de 5 mm.
- Cache das listas de colisão por estado da comporta. A interação percorre os sólidos sem alocar cópia.
- Validação estendida para diretor, retirada do vulto fora do quadro, ilusão, registro do Controle, pausa e reinício.

## v0.0.7 — Vistoria da Bomba 02

- Vistoria com duas leituras, rearme, volta à nave e liberação da saída.
- Dez pontos de aparição, rastro na bacia, pancadas na Bomba 02 e câmera fixa consultável no Controle.
- Base preservada na tag local v0.0.7 e na distribuição Windows correspondente.
