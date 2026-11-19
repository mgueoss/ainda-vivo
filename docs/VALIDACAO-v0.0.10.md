# Validação — v0.0.10 Ecos do Desaparecimento

**18 de novembro de 2026 · Windows x64 build 26200 · Ryzen 5 5600G · GeForce GTX 1660 SUPER · Vulkan, driver 610.88**

Build offline do executável distribuído com Rust e Cargo 1.97.1, Bevy 0.18.1 e o perfil de desenvolvimento otimizado do projeto. Os testes gráficos usaram janela de 1280 × 720. O executável empacotado tem o mesmo hash da build que executou as rotas.

| Verificação | Resultado |
|---|---|
| Rustfmt e Clippy na workspace com `-D warnings` | Aprovados |
| Testes Rust, em dois grupos para reaproveitar o cache | 46 aprovados. São 20 no jogo, 23 no núcleo e 3 no launcher. |
| Build Windows x64 offline | Aprovada |
| Vistoria completa com renderização e áudio nativos | PASS em 12.260 quadros e 21 capturas BMP |
| Captura e Despertar com áudio nativo, seed 20261118 | PASS em 2.696 quadros e 10 capturas BMP |
| Captura e Despertar em modo mudo, seed 73133 | PASS em 2.696 quadros |
| Rota da OS 17, seed 20261118 | PASS em 2.735 quadros, seis capturas BMP e três fontes de passos em reprodução |
| Extração, verificação e abertura pelo botão JOGAR | PASS. Janela v0.0.10 com renderizador Vulkan. Verificação local de 392 ms. |
| Pacote sem executável | Bloqueado pela verificação |

## Cobertura das rotas

- **Bomba 02.** Trajeto pela física desde a oficina, fusível, primeira leitura, rearme, segunda leitura, comporta e saída. Inclui colisões, agachamento, pausa, lanterna, raios de interação, monitor, reinício, placa contraditória e ficha que volta à posição inicial. Verificou 15 setores, 22 portais, dez linhas de visão, quatro pontos de aparição efetivamente enquadrados, travessia na passarela e retirada do vulto fora de vista. O dispositivo de áudio reproduziu três famílias de incidentes e os sinais da vistoria e da bacia.
- **Continuidade e desfechos.** Após perceber a figura, o roteiro caminhou até receber aviso. Recuar evitou o contato na primeira tentativa. Na segunda, exposição sustentada durante movimento causou captura natural. O teste comparou amostras adjacentes de posição, velocidade, câmera, lanterna, relógio e mensagem. Verificou o relógio contraditório, a baixa percebida, pausa do áudio e do relógio, contradição após a baixa, recuperação no posto, saída verdadeira e reinício. A aparição repetiu o tempo com a mesma seed. O teste com som também confirmou reprodução do aviso e da portaria.
- **OS 17 e retomada.** O roteiro posicionou o operador perto das interações e usou **E** para ler a nota, ligar e apagar a mesa, rearmar o quadro, consultar o relé e abrir as duas imagens do monitor. Caminhou para gravar a cadência e conferiu três fontes de eco com posição de reprodução positiva no dispositivo nativo. Confirmou a observação da ficha e do relé, a transferência da ficha, a troca fora de visão, os horários 03:19 e 03:17, a possibilidade de concluir a vistoria e uma única retomada da tentativa após Despertar vivo.
- **Regras isoladas.** Os testes novos conferem observação interrompida, variantes lembradas, isolamento entre observadores, revisita, dependência entre os acontecimentos e duração das janelas. O teste da baixa reconciliada prova que a contradição vem após 5,5 s e que o relé, a gravação e a baixa percebida preservam a trava autoritativa. O teste do núcleo cobre retomada com segunda leitura real já registrada e limpeza da consulta anterior do monitor.

[Saída da vistoria](smoke-v0.0.10-rota.txt) · [Saída da continuidade com áudio](smoke-v0.0.10-realidade.txt) · [Saída da segunda seed](smoke-v0.0.10-seed-alternativa.txt) · [Saída da OS 17](smoke-v0.0.10-os17.txt)

## Intervalos reais de quadro

No trecho de caminhada e espera da rota da OS 17, com áudio ligado e VHS desligado, o relógio monotônico registrou **699 intervalos** entre quadros do aplicativo. Mediana **6,08 ms** e percentil 95 **6,72 ms**. A coleta exclui carregamento, outras salas e os quadros solicitados para captura de imagem. O relógio de jogo do smoke continua fixo em 1/60 s. Os números descrevem um trecho específico em condições controladas. Uma sessão livre pode apresentar intervalos diferentes.

## Integridade e dependências

| Arquivo | Bytes | SHA-256 |
|---|---:|---|
| `ainda-vivo-v0.0.10-windows-x64.zip` | 24.494.798 | `f727b188d4df659ba25c9eb98c2631c9bdc93c36c50345bdc0aae186e6d2cd59` |
| `game/ainda-vivo.exe` | 71.289.856 | `5fa3dc4b8d27ac15431bc3cbaefedf7234580a0954d36b10042ab9065ae64938` |
| `AINDA-VIVO.exe` | 467.968 | `ad6a48add6355478cdf510c58d83f6aaffe0a01a91fb94cc1480b7b6620bb0d3` |

O PE do jogo depende dos componentes do Windows e de `VCRUNTIME140.dll`. O launcher universal pré-compilado tem runtime estático e conserva o SHA-256 das distribuições anteriores. O teste abriu o jogo pelo botão nativo, depois de validar o ZIP e o executável extraído. O pacote inclui os avisos das licenças. O runtime Visual C++ e o driver já estavam disponíveis neste PC. [Como o launcher verifica e oferece reparo](DISTRIBUICAO.md)

## Capturas reais da rota da OS 17

- [Luminária ligada e segunda via no Arquivo](imagens/v0.0.10/01-arquivo-luminaria.png)
- [Relé cancelado](imagens/v0.0.10/02-rele-cancelado.png) · [Relé assinado e ficha deslocada](imagens/v0.0.10/03-rele-assinado.png)
- [Quadro original da Câmera 02](imagens/v0.0.10/04-camera-original.png) · [Quadro das 03:17](imagens/v0.0.10/05-camera-0317.png)
- [Retomada oferecida após Despertar](imagens/v0.0.10/06-retomada-oferecida.png)

## Limites

As rotas automatizadas exercitam o jogo Windows e o dispositivo de áudio. A medição cobre uma máquina e um trecho específico. Avaliação humana de tensão, legibilidade, repetição e conforto ainda está pendente. Também falta validar o reparo real do Visual C++ em uma instalação limpa do Windows.
