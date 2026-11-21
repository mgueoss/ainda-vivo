# Validação v0.0.11 · Colapso

Testes executados em Windows x64, com GTX 1660 SUPER, Vulkan e driver 610.88. Build de desenvolvimento otimizada, Rust e Cargo 1.97.1, Bevy 0.18.1 e cache local.

## Núcleo e compilação

Rustfmt e Clippy com `-D warnings` passaram. A build offline e os **60 testes Rust** passaram, com 24 no jogo, 33 no núcleo e 3 no inicializador. Os testes do núcleo cobrem os seis níveis, limites, histerese, simultaneidade, orçamentos, imunidade a realimentação, descanso, prazo, último segundo e partições de passos. Os testes de apresentação conferem o movimento da despedida, o ganho de ambiência e as amostras estéreo a 24 kHz da respiração e do pulso.

## Rotas nativas

As rotas de integração usam tempo virtual fixo de 1/60 s com o aplicativo, o renderizador e a física Windows. Os comandos são roteirizados. A entrada no posto e o uso de **R** passam pelos sistemas normais de interação. O roteiro injeta pressão para alcançar as janelas críticas de forma repetível.

| Rota | Cobertura |
|---|---|
| `--smoke-collapse --check-audio` | PASS, 8.548 quadros. Posto e reserva, pausa, chegada a pé ao refúgio com 56,45 s, independência da captura, uso de R com 0,75 s na falsa realidade, reinício, expiração, ajoelhamento e despedida. |
| `--smoke-menu` | PASS. Abertura e pausa de 1280 × 720 a 640 × 480, efeito audiovisual, câmera de crise, indicador oculto por padrão, Despertar e retomada. |
| `--smoke-reality` | PASS, 2.696 quadros. Recuo, continuidade da captura, realidade percebida, saída falsa, Despertar, saída verdadeira e reinício. |
| `--smoke-echoes` | PASS, 2.735 quadros. Percurso e acontecimentos da OS 17, duas imagens da Câmera 02 e retomada. |
| `--smoke-test` | PASS, 12.260 quadros. Percurso completo da Bomba 02, colisão, interações, manômetro, rearme, travessia, Presença, visor e conclusão. |

No smoke de Colapso, as fontes de respiração e batimentos avançaram no dispositivo nativo e pausaram junto da sessão. Em outras inicializações o Windows retornou `0x80070490` para a saída padrão. A comparação de desempenho e as rotas de regressão da vistoria, captura e OS 17 foram executadas com o som silenciado. [Relatório de Colapso](smoke-v0.0.11-colapso.txt) · [Relatório de Acessibilidade](smoke-v0.0.11-acessibilidade.txt) · [Vistoria completa](smoke-v0.0.11-vistoria.txt)

## Medição comparável

A mesma rota automatizada de realidade, com 2.696 quadros, seed 20261118, áudio mudo e sem solicitação de capturas, levou **17,75 s** na v0.0.10 com menu revisado e **17,83 s** na v0.0.11. São tempos de parede de uma execução de cada versão no PC de teste, incluindo inicialização. A rota opcional da OS 17 na v0.0.11 mediu 699 intervalos reais em caminhada e espera, com mediana de 6,06 ms e p95 de 6,66 ms. A coleta descreve esses recortes automatizados. [Rota da v0.0.10](smoke-v0.0.11-referencia.txt) · [Rota da v0.0.11](smoke-v0.0.11-realidade.txt) · [Medição da OS 17](smoke-v0.0.11-ecos.txt)

## Balanceamento inicial

Simulei até 240 segundos de estímulos para cada perfil diretamente no modelo a 60 Hz. As linhas representam agendas de estímulos e recuperação, com intensidades escolhidas para explorar limites.

| Perfil simulado | Pico de instabilidade | Resultado |
|---|---:|---|
| Cauteloso, com pausa | 25,3 | Sem crise |
| Curioso, com anomalias e refúgio | 36,9 | Sem crise |
| Corrida com avistamentos espaçados | 19,3 | Sem crise |
| Ignora o entorno e escuta poucos ruídos | 7,6 | Sem crise |
| Perdido após um avistamento | 9,1 | Sem crise |
| Observação intensa, com contato e sinais fortes | 100,0 | Uma crise expirou |
| Mesma exposição intensa, com reserva | 100,0 | Reserva no último segundo, recuperação para 70 |

A recepção do horror, a frequência orgânica de colapso e o conforto ainda precisam de avaliação em partidas humanas. [Modelo e estratégia de calibração](../AINDA-VIVO-HANDOFF-v0.0.11.md) · [Resultados dos perfis](smoke-v0.0.11-equilibrio.txt)

## Capturas do jogo

- [Posto de pausa](imagens/v0.0.11/01-posto-de-pausa.png) · [Indicador de urgência](imagens/v0.0.11/02-urgencia-opcional.png)
- [Figura imóvel](imagens/v0.0.11/03-presenca-imovel.png) · [Inclinação da cabeça](imagens/v0.0.11/04-inclinacao.png) · [Aceno](imagens/v0.0.11/05-aceno.png)
- [Registro encerrado](imagens/v0.0.11/06-registro-encerrado.png) · [Acessibilidade a 640 × 480](imagens/v0.0.11/06-acessibilidade-compacta.png)

## Distribuição

O ZIP foi extraído e o botão **JOGAR** abriu o executável com título v0.0.11 no Vulkan. A verificação local levou 427 ms nesta execução. O teste também bloqueou um pacote sem executável. A cópia portátil inclui o inicializador universal, avisos de licença, versão e instruções. O Visual C++ e o driver já estavam instalados na máquina de teste.

| Arquivo | Bytes | SHA-256 |
|---|---:|---|
| `ainda-vivo-v0.0.11-windows-x64.zip` | 25.044.504 | `ee9d771f723f3fafa1e442efee6b3c27b14185de9e1cadcc7cc350450ce18a81` |
| `game/ainda-vivo.exe` | 72.501.760 | `957501b8d97fa47745431103f50e47ad75ebe9abcbd0fc6e018a42b3a9969b90` |

[Release v0.0.11](https://github.com/mgueoss/ainda-vivo/releases/tag/v0.0.11) · [ZIP](https://github.com/mgueoss/ainda-vivo/releases/download/v0.0.11/ainda-vivo-v0.0.11-windows-x64.zip) · [Verificação SHA-256](https://github.com/mgueoss/ainda-vivo/releases/download/v0.0.11/ainda-vivo-v0.0.11-windows-x64.zip.sha256)
