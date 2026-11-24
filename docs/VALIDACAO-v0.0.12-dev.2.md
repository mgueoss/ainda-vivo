# Validação · v0.0.12-dev.2 — Movimentos Antinaturais

## Rig, deformação e contato

O Blender 4.5.16 LTS confirmou a preservação das posições da malha, dos pesos por vértice, dos 52 ossos e de suas posições de repouso em relação à dev.1. A superfície tem 18.800 triângulos e o GLB preserva um material e três texturas. O Bevy importou os 39 clipes, incluindo as sete entradas anteriores e 32 novas ações com metadados. O GLB passou de 2.306.904 para 3.853.596 bytes.

As ações novas foram amostradas a 60 Hz, com uma leitura entre cada par de quadros da fonte de 30 Hz. O mínimo vertical considera toda a superfície deformada. O deslizamento mede a maior variação horizontal do ponto de referência do osso durante um intervalo de apoio, compensando o deslocamento nominal em piso plano. A medida da curva cobre a pose com avanço retilíneo. [Leituras por clipe e membro](validacao/v0.0.12-dev.2/deformacao-e-apoios.json)

| Ação | Mínimo em relação ao piso | Maior deslizamento em um apoio |
|---|---:|---:|
| Ruptura lenta | +1,24 mm | 0,22 mm |
| Ruptura rápida | +0,93 mm | 1,40 mm |
| Arrancada | −1,16 mm | 4,78 mm |
| Corrida moderada | −1,36 mm | 6,20 mm |
| Corrida intensa | +0,22 mm | 5,55 mm |
| Mudança de direção | −1,24 mm | 13,87 mm |
| Desaceleração | −0,47 mm | 6,87 mm |
| Parada brusca | −0,81 mm | 6,43 mm |
| Retirada quadrúpede | +3,96 mm | 2,51 mm |

A mão direita chegou a 20,28 mm acima do piso na janela de apoio durante a curva. O pior quadro de percentil 99 de alongamento, na arrancada, ficou em 1,894×. A corrida intensa ficou em 1,793×. As medidas de apoio se referem ao deslocamento para o qual cada clipe foi autorado. Valores editados de velocidade de deslocamento e misturas entre clipes podem produzir desvios adicionais. A correção no terreno, a colisão dinâmica e a direção do tronco em cada curva são pontos para a etapa de integração.

## Dev Lab e executável

Rustfmt, Clippy de todo o workspace com avisos como erros, compilação Windows offline e a verificação sem a feature `dev-lab` passaram. Foram **68 testes Rust** aprovados, incluindo 32 do jogo, 33 do núcleo e três do inicializador. Os novos testes cobrem tempos, cruzamento de marcadores, comportamento em diferentes taxas de atualização, limites da rota, postura e orientação para o operador. [Saída dos testes](validacao/v0.0.12-dev.2/rust.txt)

O executável extraído do ZIP passou **1.520 quadros** no teste nativo de movimentos, com **112 eventos**. A pista percorreu **9,502 m** e alcançou **0,368 rad** de rotação. A demonstração em primeira pessoa terminou a **2,662 m entre inserções** do modelo e da câmera. O roteiro também verificou `Ctrl+I`, congelamento, avanço e retorno de quadro, extremos de velocidade, mistura, interrupção, A/B, guias, acompanhamento do operador e seleção da sequência. [Saída do teste de movimentos](validacao/v0.0.12-dev.2/movimentos.txt)

O modo de GLB externo e a recarga passaram em **4.101 quadros**, incluindo **32 alternâncias adicionais** com a campanha pausada. As contagens de entidades, malhas, materiais e imagens ficaram estáveis entre as saídas equivalentes. O atalho do ZIP abriu a janela identificada como v0.0.12-dev.2. O SHA-256 do ZIP da dev.1 foi conferido após o empacotamento da dev.2. [Estresse e GLB externo](validacao/v0.0.12-dev.2/lab-glb-e-estresse.txt) · [Atalho](validacao/v0.0.12-dev.2/atalho.txt)

| Arquivo | Bytes | SHA-256 |
|---|---:|---|
| ZIP Windows x64 experimental | 30.216.876 | `ba3aa2f9c6bdb586e6d820cc9ea97016be9947906c4e935f7f43b287218f4cfd` |
| Executável extraído e testado | 85.225.472 | `137dfda538331ecff5f7f7ab47a33070b5acebfe670073be3a02e41d777830a7` |

## Campanha e comparação de desempenho

O Windows usou a GTX 1660 SUPER, driver 610.88 e Vulkan. O roteiro original do Dev Lab executou 1.492 quadros por repetição na ordem dev.1, dev.2, dev.2 e dev.1, com configurações novas e sem exportar capturas. A mediana de estúdio usa 25 intervalos de parede por execução. O tempo total inclui abertura e encerramento. [Medições](validacao/v0.0.12-dev.2/tempos.json)

| Build | Medianas de estúdio | Tempos totais |
|---|---|---|
| v0.0.12-dev.1 | 6,0715 e 6,0969 ms | 10,118 e 10,123 s |
| v0.0.12-dev.2 | 6,2384 e 6,0124 ms | 10,100 e 10,138 s |

A diferença média de 0,0412 ms entre as medianas ficou dentro da variação das duas execuções da dev.2. O recorte compara a mesma cena de inspeção nas duas versões. Ele informa o custo do Lab em condições equivalentes. A corrida, misturas e guias em uso têm seu próprio custo.

- A vistoria da Bomba 02 passou em 12.260 quadros, com dispositivo de áudio verificado. O teste registrou três sons incidentais distintos e a reprodução do som de vistoria e do som do reservatório. [Vistoria e áudio](validacao/v0.0.12-dev.2/vistoria-audio.txt)
- Saída verdadeira, captura silenciosa, Despertar e continuidade dos controles e da lanterna passaram em 2.696 quadros. [Realidade](validacao/v0.0.12-dev.2/realidade.txt)
- A investigação opcional da OS 17 e a Câmera 02 passaram em 2.735 quadros. [Ecos](validacao/v0.0.12-dev.2/ecos.txt)
- As rotas de recuperação, expiração e despedida da campanha passaram em 8.545 quadros. [Colapso](validacao/v0.0.12-dev.2/colapso.txt)

## Inspeção e capturas

As imagens são quadros reais da janela Bevy a 1280 × 720. A apresentação de laboratório mantém a luz configurável. O MP4 foi reaberto e conferido com 180 quadros consecutivos da aproximação em primeira pessoa a 30 Hz.

| Inspeção | Arquivos |
|---|---|
| Preparação e transferência de peso | [Ruptura](imagens/v0.0.12-dev.2/01-ruptura.png) · [Mãos no chão](imagens/v0.0.12-dev.2/02-maos-no-chao.png) |
| Corrida e contato | [Ciclo intenso](imagens/v0.0.12-dev.2/03-passada-quadrupede.png) · [Pista](imagens/v0.0.12-dev.2/05-pista.png) |
| Iluminação durante a aproximação | [Vista em primeira pessoa](imagens/v0.0.12-dev.2/06-aproximacao-lanterna.png) · [Vídeo](imagens/v0.0.12-dev.2/aproximacao.mp4) |
| Interpretação humanoide | [Despedida](imagens/v0.0.12-dev.2/04-despedida.png) · [Banco](imagens/v0.0.12-dev.2/08-banco.png) |
| Inspeção dos clipes | [Linha do tempo e guias](imagens/v0.0.12-dev.2/07-linha-do-tempo.png) |

A inspeção visual concentrou-se na ruptura, na passada intensa, na pista, na despedida, na postura sentada e na aproximação sob a lanterna. O catálogo restante tem amostragem de deformação no Blender. As capturas permitem avaliar leitura e enquadramento. Peso percebido, conforto e efeito de terror dependem da avaliação em jogo pelo diretor e pelos testers.
