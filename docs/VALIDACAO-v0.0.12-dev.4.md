# Validação · v0.0.12-dev.4 — A Caçada

Rustfmt, Clippy com avisos como erros, compilação offline e verificação sem a feature do Dev Lab passaram. As suítes Rust passaram **78 testes**. [Saída Rust](validacao/v0.0.12-dev.4/rust.txt)

O Windows executou o roteiro do ZIP extraído em **1.390 quadros**, com seed 20261118. A corrida livre chegou a **4,95 m/s**. A perseguição da Estação despachou **14 marcadores de mão e 12 de pé** e a captura preservou a apresentação do jogador. A porta, a ameaça, o isolamento do contato no Lab e a retomada da missão passaram. [Saída do roteiro](validacao/v0.0.12-dev.4/cacada.txt)

| Artefato | Bytes | SHA-256 |
|---|---:|---|
| ZIP Windows x64 | 30.352.189 | `359d6bb5e29359a04af6d7f5be27c37043afc8201931795bd8485fb71e98e13f` |
| Executável testado | 85.596.160 | `2298ab664455312991a954632ce79de5d152ce798d39f80c8315294c80039e14` |

## Roteiros

- As suítes exercitam arrancada, tempo limite, colisão com a porta, perda de rastro, blefe, proteção, envelopes de animação, possibilidade de fuga com o sprint atual de 4,5 m/s e os cinco desfechos. O teste do diretor também confere o intervalo, a fase de missão, a instabilidade e a exigência de saída disponível.
- O smoke específico monta um encontro reproduzível na Estação real e força a seleção do desfecho de captura. O motor da perseguição ainda precisa alcançar o jogador por um segmento desobstruído. O teste verifica mudança silenciosa de realidade, continuidade de posição, câmera e lanterna, preservação da vistoria e retorno pelo Despertar. O mesmo roteiro abre o Dev Lab para ensaiar contato, pressão psicológica, blefe e navegação com a mudança de porta.

## Capturas

- [Caçada na Estação](imagens/v0.0.12-dev.4/01-cacada-na-estacao.png)
- [Corrida e controles do Dev Lab](imagens/v0.0.12-dev.4/02-cacada-livre.png)
- [Porta e recuo](imagens/v0.0.12-dev.4/03-porta-e-recuo.png)

Os quadros são da janela Bevy a 1280 × 720. A taxa exibida pelo HUD do smoke segue o relógio fixo da simulação.

## Comparação com a dev.3

O cenário de estúdio do Dev Lab executou 1.492 quadros a cada repetição, na mesma GTX 1660 SUPER com Vulkan. As medianas usam 25 intervalos de parede. [Tempos e adaptador](validacao/v0.0.12-dev.4/tempos.json)

| Build | Medianas do estúdio | Tempo total do roteiro |
|---|---:|---:|
| dev.3 | 6,1505 e 6,1110 ms | 10,151 e 10,129 s |
| dev.4 | 6,1008 e 6,2165 ms | 10,130 e 10,113 s |

As medianas se sobrepõem entre as versões. O estúdio mede a mesma cena, com a caçada desativada.

Com caçada ativa, o roteiro específico teve 1.390 quadros em **4,121 s de parede**, incluindo abertura e encerramento. O máximo de memória residente observado nas 15 amostras foi **475,52 MiB** para o processo inteiro. O roteiro usa relógio fixo de 60 Hz e roda sem gravação nesta medida. [Medição](validacao/v0.0.12-dev.4/medicao.json)

## Preservação

- Observador e corredor de IA em 2.880 quadros. [Observador](validacao/v0.0.12-dev.4/observador.txt)
- Pista e animações anteriores em 1.520 quadros. [Movimentos](validacao/v0.0.12-dev.4/movimentos.txt)
- Vistoria em 12.260 quadros com dispositivo de áudio verificado. [Vistoria](validacao/v0.0.12-dev.4/vistoria-audio.txt)
- Realidade, OS 17 e Colapso passaram em 2.696, 2.735 e 8.545 quadros. [Realidade](validacao/v0.0.12-dev.4/realidade.txt) · [OS 17](validacao/v0.0.12-dev.4/ecos.txt) · [Colapso](validacao/v0.0.12-dev.4/colapso.txt)

## Limitações de avaliação

As condições físicas são verificadas em rotas reais e em cenários reproduzíveis. A frequência natural de caçadas, o susto e a leitura de peso em percursos variados dependem de testes humanos. A navegação quadrúpede conserva espaço para voltar à postura humana e pode desistir de um vão estreito que o jogador atravessa. A mistura das ações usa a velocidade real. Curvas apertadas e mudanças de passada podem ter deslizamento residual. Os recursos sonoros desta etapa são temporários e os testes automatizados registram os marcadores, não a impressão de quem ouve.
