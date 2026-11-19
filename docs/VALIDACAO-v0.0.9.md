# Validação — v0.0.9 Falsa Realidade

**Windows x64 · 18 de novembro de 2026 · GTX 1660 SUPER, driver 610.88, Vulkan**

O executável foi compilado com Rust/Cargo 1.97.1 no perfil de desenvolvimento otimizado do projeto. Reutilizei o cache de bibliotecas do PC. Nenhuma dependência nova foi acrescentada. Os testes encadearam o relógio fixo do jogo, o aplicativo Windows e o dispositivo sonoro quando solicitado.

| Verificação | Resultado |
|---|---|
| Formatação e Clippy da workspace com `-D warnings` | Aprovados |
| Testes Rust offline | 41 aprovados. São 19 no jogo, 19 no núcleo e 3 no inicializador. |
| Compilação do jogo Windows offline | Aprovada |
| Rota de regressão com áudio | PASS · 12.260 quadros, 21 capturas BMP, nenhuma falha |
| Continuidade com seed 20261118 e áudio | PASS · 2.696 quadros, 10 capturas BMP, nenhuma falha |
| Continuidade com seed 73133 em modo mudo | PASS · 2.696 quadros, nenhuma falha |
| ZIP portátil extraído e aberto com o inicializador | PASS · título v0.0.9, Vulkan e verificação local de 399 ms |

## O que foi validado

- **Rota completa.** Fusível, primeira leitura, rearme, pressurização, segunda leitura, comporta, Decantação e saída. O script testou também colisão, agachamento, lanterna, interação pelo raio, gravação do Controle, retorno da oficina, passagem do vulto, pausa, retomada e reinício. A auditoria conferiu 15 setores, 22 portais e 10 linhas de visão. Incidentes sonoros e os dois sons autorais da vistoria tiveram reprodução verificada.
- **Captura e recuo.** O operador viu a figura no vão oeste e avançou com a física até a zona de aviso. Recuar protegeu a primeira tentativa. Na segunda, manter o olhar afastado e deslocar-se devagar causou captura natural. O teste comparou amostras adjacentes do corpo, velocidade, yaw, pitch, lanterna, relógio e mensagem. A rota continuou com os mesmos controles.
- **Desfechos e recuperação.** Um segundo registro privado abriu a comporta no visor. A travessia mostrou a baixa e a contradição. A pausa congelou a espera e pausou os sons. Despertar devolveu o operador ao posto, restaurou a trava verdadeira e permitiu refazer apenas a segunda leitura. Cruzar após o registro real concluiu a vistoria. Confirmar Despertar vivo interrompeu a tentativa. O reinício restaurou o fusível, o participante e os estados privados.
- **Anomalias e direção.** A ficha e o indicador da v0.0.8 continuam acionáveis. O relógio mudou após o olhar virar. Foi verificada a leitura da nova instrução pela interação comum. A rota de regressão visitou os seis modos do diretor. A rota de continuidade repetiu a hora da aparição dentro de cada seed. A provação de captura usa as condições reais do jogo. O smoke pede uma janela de ilusão para garantir a oportunidade do relógio.
- **Áudio.** Sinks do aviso de contato e do som na portaria avançaram no dispositivo nativo. Passos em movimento e silêncio em repouso foram conferidos pelos testes de emissão. A segunda seed conferiu a lógica em modo mudo, sem alegar teste de reprodução naquela execução.

[Relatório completo da rota](smoke-v0.0.9-route.txt) · [Relatório de continuidade](smoke-v0.0.9-reality.txt) · [Relatório da segunda seed](smoke-v0.0.9-alternate.txt)

## Integridade da distribuição

| Arquivo | Bytes | SHA-256 |
|---|---:|---|
| `ainda-vivo-v0.0.9-windows-x64.zip` | 24.460.824 | `00d1bfa9342c07d467c8fee34ddf4fd4f90f8446c7123bb86a9ba125946816b5` |
| `game/ainda-vivo.exe` | 71.188.992 | `979fdf9914446e9f21fb842dd94d930fee92c18cd7a3ae6bf358dbea3762c8bb` |
| `AINDA-VIVO.exe` | 467.968 | `ad6a48add6355478cdf510c58d83f6aaffe0a01a91fb94cc1480b7b6620bb0d3` |

O executável dentro do ZIP coincide com o binário usado nos três smokes. O launcher tem o mesmo SHA-256 da distribuição v0.0.8. O empacotador incluiu os avisos de licença existentes e o teste bloqueou uma cópia sem executável. A validação de 399 ms mede só o preparo do launcher nesta máquina. [Guia de distribuição](DISTRIBUICAO.md)

## Capturas Windows

- [Aparição no retorno com VHS desligado](imagens/v0.0.9/retorno-observado.png) · [Continuidade após o contato](imagens/v0.0.9/continuidade-apos-contato.png)
- [Relógio às 02:48](imagens/v0.0.9/relogio-antes.png) · [Relógio às 03:12](imagens/v0.0.9/relogio-depois.png)
- [Baixa da portaria](imagens/v0.0.9/portaria-baixa.png) · [Contradição da portaria](imagens/v0.0.9/portaria-pendente.png)
- [Confirmação de Despertar](imagens/v0.0.9/confirmacao-despertar.png) · [Posto retomado](imagens/v0.0.9/posto-retomado.png)
- [Saída verdadeira](imagens/v0.0.9/saida-real.png) · [Registro interrompido](imagens/v0.0.9/registro-interrompido.png)

## Limites da validação

As rotas usam tempo virtual e entradas controladas. A rota de continuidade posiciona o operador para interações específicas. Ela percorre fisicamente a aproximação, o recuo, o movimento durante a captura e a travessia da comporta. Os quadros do smoke não medem FPS real. Conforto subjetivo, tensão e taxa de captura em partidas livres precisam de avaliação humana. O Visual C++ já estava instalado no PC. O fluxo de reinstalação do componente em uma máquina limpa permanece pendente de validação.
