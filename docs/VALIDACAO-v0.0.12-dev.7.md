# Validação da v0.0.12-dev.7

Em 26 de novembro de 2026, a suíte rodou no Windows 11 build 26200, Ryzen 5 5600G, 16 GB de RAM, GTX 1660 SUPER, driver 610.88, Vulkan e janela de 1280×720. As duas compilações do jogo usaram o perfil `dev` otimizado. Os testes de Rust usaram as dependências travadas e o cache offline. O inicializador Win32 foi compilado com CRT estático.

## Ferramentas e regressões

- `rustfmt` e Clippy com `-D warnings`, com todas as features e na edição comum, passaram. A suíte do workspace passou com 65 testes do jogo, 33 do núcleo e 4 do inicializador. Os dois executáveis do jogo foram compilados. A edição comum percorreu toda a missão física na seed 20261118, de entrada no menu a `Escaped`.
- A matriz cognitiva de regressão usa 9 cenários em 30, 60 e 120 Hz. Passaram as 108 combinações da suíte Rust e as 135 da execução com seed adicional. Os ensaios avaliam deslocamento lateral, curva, obstáculos, bloqueio, percepção, reabertura de rota, distração, interrupção e uma sessão longa.
- Nos colisores reais, as 17 varreduras baixas deram passagem zero nas bocas Acesso–Carga e Acesso–Arquivo. O teste A* encontrou uma rota em pé nas duas. Um ensaio separado executou a frenagem e o cruzamento em pé, sem contato ou atravessamento, em corredor de geometria controlada.
- Preferências em Windows. `settings.cfg` ficou aberto sem permissão de substituição. O jogo registrou falhas com retentativas de 1 s e 2 s, preservou `fov=70` e gravou toda a configuração no mesmo processo após liberar o arquivo. Com 12 CSVs de teste e mais a execução nova, a retenção deixou oito sessões reconhecidas e preservou `reserva.csv`. O limite de 180.000 linhas é aplicado na escrita.
- O ZIP foi extraído, os SHA-256 foram conferidos e os dois atalhos do Windows executados. O inicializador detectou os requisitos, abriu a versão esperada em Vulkan nos dois modos e bloqueou a amostra sem executável. O modo do Lab recebeu `--dev-lab`; o normal abriu sem o argumento. Nesta máquina o runtime Visual C++ x64 já estava disponível.

## Roteiros independentes de campanha

O smoke legado executa as verificações destinadas à silhueta. A auditoria encontrou três falhas específicas desse ator quando o roteiro era executado com a criatura nova, uma falha de janela de ilusão visual e outra de contagem de imagens. As três verificações de silhueta e a ilusão continuam no roteiro legado. O roteiro físico valida a missão no ator novo. A cadeia opcional da ficha, do relé e da Câmera 02 tem seu próprio smoke, com verificação de apresentação e preservação dos objetivos. Os ensaios de saída, captura, colapso e Dev Lab têm verificações separadas.

Os identificadores originais eram `presence_and_basin_on_walking_route`, `director_main_route_and_crossing`, `presence_and_unseen_retirement`, `illusion_does_not_shift_behind_observer` e `capture_completion`. Permanecem em `src/smoke.rs`. O teste de observação em Windows mede aparição real e deslocamento do ator físico. No percurso legado com gravação, a contagem exigida de 21 foi satisfeita.

| Smoke Windows | Resultado |
|---|---|
| Roteiro legado com áudio e capturas | PASS, 12.260 quadros, 21 capturas, ficha alterada e restaurada no reinício. |
| OS 17 | PASS, 2.735 quadros, três fontes de passos e a sequência narrativa concluída. |
| Falsa realidade | PASS, 2.696 quadros, captura preservou os controles e Despertar recuperou o posto no percurso de continuidade. |
| Refúgio e colapso | PASS, 8.545 quadros, dois sinais de áudio e cinco fases da despedida. |
| Caçada física controlada | PASS, 2.055 quadros, 14 apoios de mãos, 12 de pés, captura silenciosa, continuação da missão e busca com porta fechada. |
| Observador | PASS, 2.880 quadros, uma manifestação física e validação de colisão e percepção. |
| Laboratório cognitivo | PASS, 554 quadros, estalo no relé observado após 1,483 s e desvio de 1,44 m do bot no ensaio. |
| Áudio | PASS, dispositivo nativo, porta fechada com ganho de 0,046 contra 0,090 aberta, pico de sete fontes, fontes encerradas e campanha preservada. |
| Dev Lab | PASS, 1.492 quadros, cena, controles e ensaios de animação. |

### Campanhas completas com IA ativa

Entrada pelo menu, deslocamento pelo motor normal, interações pelo raio de **E** e quatro marcos da vistoria. O roteiro define comandos e direção do jogador. A instância da criatura fica sob seu sistema habitual, com o diretor e as ilusões sonoras ativos. Os perfis alteram rotas, movimento ou esperas. A taxa da simulação indicada na tabela é programada para o ensaio e independe da cadência gráfica medida.

| Perfil | Seed | Hz | Jogo (s) | Manifestações | Planos | Caçadas | Desfecho |
|---|---:|---:|---:|---:|---:|---:|---|
| Corrida | 20261118 | 60 | 59,283 | 1 | 2 | 0 | Escaped |
| Cauteloso | 17 | 30 | 95,773 | 2 | 0 | 0 | Escaped |
| Explorador | 42 | 120 | 112,243 | 1 | 1 | 0 | Escaped |
| Distração | 711 | 60 | 64,333 | 1 | 2 | 0 | Escaped |
| Repete rota | 318 | 60 | 85,570 | 1 | 3 | 0 | Escaped |
| Muda o comportamento | 4141 | 60 | 87,187 | 2 | 2 | 0 | Escaped |
| Explorador extra | 3 | 60 | 112,776 | 2 | 0 | 0 | Escaped |
| Explorador extra | 19 | 60 | 112,776 | 2 | 1 | 1 | Escaped |
| Explorador extra | 61 | 60 | 112,776 | 2 | 0 | 0 | Escaped |
| Explorador extra | 77 | 60 | 112,776 | 1 | 1 | 0 | Escaped |
| Explorador extra | 167 | 60 | 112,776 | 2 | 0 | 0 | Escaped |

Nas 11 rotas, a Presença manifestou-se 17 vezes e iniciou 12 planos. Houve 10 investigações, uma falsa retirada e uma perseguição enganosa. A janela da caçada física foi aceita uma vez, na seed 19 aos 36,10 s a 15,3 m. A corrida começou aos 38,13 s, houve um único contato e a missão terminou pela saída. Na rota rápida da seed 20261118, a falsa caçada pediu som aos 38,33 s. O diretor reservou a fonte no setor 13 aos 38,42 s, com o ator no setor 6, e emitiu o primeiro evento aos 38,67 s. O retorno do áudio levou ID e estado da entrega. A falsa retirada da seed 42 expirou após dois passos do plano aos 63,65 s.

As recusas predominantes foram ator ausente (1.448 amostras), ausência de visão confirmada (891), atenção/proteção (387), plano ou movimento ocupado (124) e postura (90). Houve ainda 11 recusas por tempo, 10 por alcance, seis por ritmo e uma por sorteio. Isso localiza a restrição nas condições geométricas e de ritmo na amostra. O total depende da duração e da amostragem a cada 0,35 s. É cedo para estimar frequência em partidas humanas. A auditoria anterior observou zero caçadas espontâneas em três rotas originais, com outro protocolo.

## Ensaios estratégicos A–I

| Comprovação | Dado observado |
|---|---|
| A. Aprender uma travessia | Sem memória o caso escolheu ameaça enganosa. Quatro cruzamentos fizeram a interceptação passar à frente. |
| B. Reveses | Duas interrupções de rota reduziram a utilidade de interceptação de 0,778 para 0,360. |
| C. Mudança de hábito | Mantendo ator e pista nas mesmas posições em uma arena simétrica, a passagem escolhida mudou de 1 para 2 após observações e reveses. |
| D. Desvio | O mesmo pedido foi julgado `Success`, `Ignored` ou `Unexpected`, conforme a reação apresentada ao sensor sintético. A ameaça sonora também foi testada com reação ignorada. |
| E. Emboscada | O motor caminhou de (2,90; −3,55) até o ponto coberto (6,10; −4,00), sem falha de varredura. Terminou `NoEvidence` depois de esperar. |
| F. Contrajogo | Sem alterar a visão, o ruído (3,30; −5,20) gerou pista aproximada (3,20; −4,80) e iniciou investigação. A rota com relé também verificou o acionamento por **E**. |
| G. Limitação perceptiva | Duas posições ocultas diferentes produziram as mesmas decisões e nenhum registro de visão. Somente a emissão física mudou a investigação. A seleção do áudio verifica setor diferente do ator e a confirmação da entrega carrega apenas estado e ID. |
| H. Frequência | 11 campanhas naturais concluídas. Houve um início físico espontâneo com contato e uma emissão estratégica de ameaça sonora. |
| I. Regressão | Matriz a três taxas, os dois portais reais, a passagem em pé, os cinco contatos possíveis e o roteiro nativo de perseguição controlada foram exercitados. |

Falsa retirada e investigação também chegaram aos seus pontos em motores headless e terminaram sem evidência. Os resultados de reação de D usam pistas simuladas. Os casos de emboscada, retirada e investigação com deslocamento usam a física do ator, em ambientes de teste. A campanha demonstrou investigação, falsa retirada e ameaça sonora. Exposição natural ao desvio, à interceptação e à emboscada ainda carece de amostras.

Reproduza A, B, D, E, F e G com `--check-strategy`. Os casos com posição fixa em C e os deslocamentos de retirada e investigação rodam em `cargo test --locked --offline --features dev-lab strategy::trials -- --nocapture`. Os resultados com a seed e as posições esperadas são registrados pela própria suíte.

### Comparação dos perfis de caçada

Ensaios de cenário a 60 Hz com fuga a 4,50 m/s. O tempo inclui a preparação. As três configurações terminaram por perda de contato no cenário que fecha a porta e mantiveram a varredura do corpo sem atravessamento nessa situação.

| Perfil | Velocidade / aceleração | Ataque / corrida e busca | Corredor livre | Porta fechada |
|---|---|---|---|---|
| Base | 4,95 m/s · 4,20 m/s² | 7,5 s · 12 s | Limite de tempo em 9,73 s | Perda de rastro em 6,05 s |
| Moderada | 5,50 m/s · 5,00 m/s² | 12 s · 18 s | Contato em 12,82 s | Perda de rastro em 6,05 s |
| Intensa | 6,00 m/s · 6,00 m/s² | 16 s · 25 s | Contato em 9,27 s | Perda de rastro em 6,05 s |

A campanha continua com Base. O corredor permite comparar resposta e fuga. O conforto e a sincronização estética dos perfis mais rápidos requerem revisão em jogo.

## Comparação gráfica com a auditoria

Rota física de corrida, seed 20261118, simulação de 60 Hz, GTX 1660 SUPER e mesma resolução. A janela de medição começa depois dos primeiros 4 s no jogo. Foram usados 3.331 quadros da dev.6 e 3.316 na dev.7, em 59,499 e 59,283 s de tempo simulado respectivamente. A dev.6 registrou capturas durante o benchmark e a dev.7 registrou apenas relógio de parede, sem gravação de imagens nesses dois ensaios. A diferença de rota e de instrumentação limita a comparação.

| Versão e modo | Média (FPS) | 1% low (FPS) | Mediana (ms) | P95 (ms) |
|---|---:|---:|---:|---:|
| dev.6 com VSync | 165,0 | 102,6 | 6,032 | 6,579 |
| dev.7 com VSync | 165,00 | 144,93 | 6,038 | 6,450 |
| dev.6 sem VSync | 461,8 | 166,7 | 2,088 | 2,772 |
| dev.7 sem VSync | 449,57 | 232,96 | 2,155 | 2,901 |

A cadência com VSync ficou próxima da versão auditada. Na execução dedicada sem VSync, a média ficou 2,6% abaixo da dev.6. A rota de diagnóstico com rastreio, executada em outro momento na mesma build, marcou 481,65 FPS, o que ilustra a variação entre rodadas. Os números são intervalos da aplicação inteira com renderizador e áudio presentes. Tempo de CPU exclusivo da IA ou de GPU por quadro, consumo de RAM e VRAM, e cauda de quadros em caçada ativa ficaram fora desta comparação. A matriz sem renderização é uma medida separada.

Na matriz sem renderização, 251.600 atualizações levaram 37,2 ms na dev.7 e 35,7 ms na dev.6. O pico do cérebro, rota e buffers das pequenas arenas passou de 10.120 para 10.912 bytes. As estruturas estáticas da Estação e a engine ficam fora dessa contagem.

## Arquivos

O pacote de entrega inclui o executável de jogo validado. As evidências de Windows ficam na pasta local `evidencias-v0.0.12-dev.7` ao lado do ZIP. O SHA-256 do portátil é `64d0ca37179742a6775dc7b6b61d3973a145fe91725048a14faf3d45ca6a700b`. O executável de jogo é `e63f89e04485dc11a4d67ea0275af2e1c9f47932f7fc8e44ab6b13a57d666142`.

As rotas, o renderizador e as saídas de áudio foram validados por automação. A avaliação da tensão, justiça, leitura da silhueta, conforto e percepção dos timbres continua com o diretor e os testers. As limitações dos assets e outras dívidas constam no [relatório consolidado](../AINDA-VIVO-RELATORIO-GUERRA-PSICOLOGICA.txt).
