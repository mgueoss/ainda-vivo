# Validação · v0.0.12-dev.3 — O Observador

Rustfmt, Clippy do workspace com avisos como erros, compilação Windows offline e compilação de verificação sem o Dev Lab passaram. Foram 73 testes Rust, dos quais 37 do jogo, 33 do núcleo e três do inicializador. [Saída Rust](validacao/v0.0.12-dev.3/rust.txt)

O executável extraído do ZIP passou o roteiro específico em **2.880 quadros** com seed 20261118. O F5 alternou nos dois sentidos. O diretor autorizou uma manifestação e a criatura percorreu **11,55 m** na Estação. A rota de inspeção consultada passou por três setores. O Lab confirmou que a porta fechada interrompeu a visão e bloqueou o trajeto direto, que a porta aberta permitiu a passagem e que a pausa congelou o relógio da criatura. [Saída do teste Windows](validacao/v0.0.12-dev.3/observador.txt)

| Arquivo | Bytes | SHA-256 |
|---|---:|---|
| ZIP Windows x64 experimental | 30.306.015 | `42bc8e833e8b97c4188dca90f3569debfaf495ffe5c76dff13a41e103ab86349` |
| Executável testado | 85.507.584 | `8c3c1780ab08190803dfe0701c9b4a230b3b1c712e42a4bd9d42523cad333ca2` |

O índice Git preserva o Blender, GLB, texturas e manifesto de animação da dev.2. O SHA-256 da distribuição anterior foi conferido após a montagem da dev.3.

## O que os testes cobrem

- Detecção após exposição contínua, perda do alvo atrás da porta, expiração de memória e audição entre setores com localização aproximada.
- Contorno de equipamento, passagem entre setores com a porta aberta, caminho bloqueado com a porta fechada e abandono de destino. O teste de espiada exige uma borda real que esconda o corpo e libere apenas o raio lateral. O teste de intimidação confirma a postura baixa sem deslocamento.
- Instância física e animação na própria Estação, acionadas por uma janela do diretor. O roteiro de navegação verifica setores de Acesso, Triagem e Bombas contra os colisores reais. Uma segunda parte testa observação, ruído, guia, pausa, porta e retorno à campanha no Dev Lab.

## Capturas no Windows

As imagens foram capturadas na janela do jogo a 1280 × 720. O modo de diagnóstico aparece no roteiro de teste. A vista da Estação usa a iluminação da campanha. O corredor de ensaio usa a iluminação do Dev Lab.

- [Presença ao fundo da Triagem](imagens/v0.0.12-dev.3/01-estacao-observador.png)
- [Porta fechada](imagens/v0.0.12-dev.3/02-porta-fechada.png)
- [Rota e painel de investigação](imagens/v0.0.12-dev.3/03-rota-investigacao.png)

## Preservação da campanha e do Lab

- O roteiro anterior do Dev Lab passou nas duas repetições da dev.3. A abertura, a volta e as contagens estáveis espelharam as verificações da dev.2. A sequência de movimentos da etapa anterior passou em 1.520 quadros. [Movimentos](validacao/v0.0.12-dev.3/movimentos.txt)
- Vistoria da Bomba 02 em 12.260 quadros, com áudio validado. [Vistoria](validacao/v0.0.12-dev.3/vistoria-audio.txt)
- Captura silenciosa original, realidade e Despertar em 2.696 quadros. [Realidade](validacao/v0.0.12-dev.3/realidade.txt)
- OS 17 e Câmera 02 em 2.735 quadros. [Ecos](validacao/v0.0.12-dev.3/ecos.txt)
- Colapso, recuperação e despedida em 8.545 quadros. [Colapso](validacao/v0.0.12-dev.3/colapso.txt)

## Desempenho

O comparativo foi executado no Windows com GTX 1660 SUPER e Vulkan. Cada repetição usou a mesma cena de inspeção da dev.2, a mesma seed e 1.492 quadros. As medianas abaixo usam 25 intervalos de parede da vista de estúdio. O cenário compara o custo do laboratório em condições equivalentes. O teste específico do Observador usa relógio fixo de simulação e valida o funcionamento da IA ativa. [Medições e adaptador gráfico](validacao/v0.0.12-dev.3/tempos.json)

| Build | Medianas do estúdio |
|---|---:|
| dev.2 | 6,0415 e 6,1095 ms |
| dev.3 | 6,0342 e 6,0412 ms |

A diferença entre as médias das medianas foi menor que a variação entre as duas repetições da dev.2. A amostra cobre o estúdio com o modo de IA desativado. Ela não estima o custo isolado de cada decisão da criatura.

Com a IA ativa no roteiro específico, os 2.880 quadros em campanha e laboratório levaram **6,723 s de parede**, incluindo abertura e encerramento. O maior conjunto residente entre 25 amostras do processo foi **472,2 MiB**. A simulação usa passo de 60 Hz e o roteiro roda sem exportar imagens. O valor de memória cobre o processo inteiro. [Medição com IA ativa](validacao/v0.0.12-dev.3/medicao.json)

## Limites conhecidos

A grade é plana e usa um pouco mais de espaço que o modelo em pé. Passagens estreitas demais para a envoltória são descartadas. A Presença respeita paredes e portas, e o jogador mantém passagem livre. A postura quadrúpede é usada como intimidação parada. A espiada depende do ângulo e da folga na borda, por isso pode ser rara em uma partida. A validação cobre a regra geométrica em teste unitário e o uso normal do clipe no Bevy. A frequência e a leitura de espionagem, acompanhamento e ameaça em partidas completas ainda precisam de avaliação humana.
