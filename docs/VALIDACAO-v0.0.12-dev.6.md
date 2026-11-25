# Validação · Inteligência Hostil

Ambiente Windows 11 Home x64 com Ryzen 5 5600G, GeForce GTX 1660 SUPER e Vulkan. A compilação usou o perfil `dev` otimizado e o cache Rust local, com dependências travadas e modo offline.

## Reprodução da falha original

| Caso | Resultado na dev.5 | Resultado após a correção |
|---|---|---|
| Operador desvia em campo aberto durante a aproximação humanoide | O estado caiu em `Follow` enquanto a aproximação deveria continuar em `Approach` | Mantém a aproximação e move o destino lateral |
| Operador desvia para junto de um equipamento durante a corrida | `End::Route` em `(0,07, -8,95)` apesar de haver uma posição de contato acessível | Chega pela margem livre e registra um contato físico |
| Desvio por trás de obstáculo retangular | A primeira matriz da dev.6 revelou um bloqueio na rotação do quadrúpede | A grade com custo de giro encontra o contorno e a varredura preserva a colisão |
| Ruído enquanto o operador está atrás da Presença | A primeira matriz da dev.6 revelou que a atenção do operador podia induzir retirada sem visão mútua | A Presença percebe o ruído físico, investiga a origem aproximada e continua sem confirmação visual do operador |

As duas primeiras regressões foram escritas e executadas antes da correção. A dev.5 também passou novamente no seu roteiro gráfico de caçada com 1.390 quadros, pico de 4,95 m/s e 14 apoios de mãos e 12 de pés na campanha.

## Cobertura de lógica

- Percepção central e periférica, queda de confiança, oclusão por porta fechada, áudio aproximado, hábitos observados, envelhecimento de pistas e verificação física do contato.
- Continuidade em desvios, reversões e circuitos. Perda de visão e retomada com porta reaberta. Desistência aceitável na passagem permanentemente bloqueada. Distração sem acesso visual ao jogador. Interrupção e simulação prolongada.
- Coordenação de janela entre ator e apresentação. Mudanças contextuais na seleção sonora. Término de uma cadeia após o operador investigar. Escolha de uma origem distinta do setor do ator físico.
- Bomba 02, captura silenciosa, estado real e falso, Despertar, sanidade e progressão cobertos pelo conjunto Rust e pelos roteiros de campanha.

O comando `--check-cognition --ai-seed 42017 --smoke-report matriz.txt` acrescenta uma seed aos quatro valores padrão. A matriz cruza nove roteiros com três taxas de atualização. O cenário prolongado percorre 180 s simulados em cada combinação. A medição de custo da matriz é tempo de relógio do processo sem renderização. O tamanho informado cobre o cérebro, o vetor de rota e os buffers A*. A grade estática da fase e a memória da engine ficam fora desse valor.

## Matriz no executável extraído

| Medida | Valor observado |
|---|---|
| Cenários aprovados | 135 de 135 com seeds 17, 42, 4141, 20261118 e 42017 |
| Taxas de atualização | 30, 60 e 120 por segundo, com simulação interna a 60 Hz |
| Desvio, reversão, circuito e contorno | 15 execuções de cada tipo com caçada válida até contato nas pistas de ensaio |
| Porta fechada e reaberta | 15 buscas com retomada após a abertura |
| Esconderijo após fechamento permanente | 15 encerramentos sem contato |
| Busca por distração, interrupção e duração prolongada | 15 aprovações de cada roteiro |
| Replanejamentos e bloqueios registrados | 1.639 e 15 |
| Retomadas após confirmação visual | 30 |
| Recuperações de movimento após bloqueio local | 0 no contador específico de retomada após travamento |
| Entradas de estado e repetições imediatas | 519 e 0 |
| Latência de primeira visão nos roteiros frontais | Até 300 ms simulados |
| Latência da investigação após ruído | Até 50 ms simulados |
| Pico do cérebro, rota e buffers | 10.120 bytes |
| Relógio da execução | 35,7 ms para 251.600 atualizações na matriz sem renderização |

## Comparação no estúdio

Ordem dev.5, dev.6, dev.6 e dev.5. Cada execução do Dev Lab completou 1.492 quadros. A mediana usa as 25 amostras de quadro do trecho de estúdio.

| Build | Medianas das duas execuções |
|---|---|
| dev.5 | 6,0741 e 6,0501 ms |
| dev.6 | 6,0192 e 5,9911 ms |

As medianas ficaram na mesma faixa nesse trecho. O roteiro cobre a cena do estúdio e a estabilidade de recursos em entradas e saídas repetidas. A matriz mede a lógica em cenários isolados.

## Laboratório Cognitivo no Windows

O teste `--smoke-inteligencia` abriu o Lab com **Ctrl+I**, colocou e removeu a caixa, ativou o bot, verificou que o avanço pausado durou 1/60 s, limpou a memória, acionou a matriz e exportou o diagnóstico. **Repetir seed** reiniciou o mesmo bot. A campanha permaneceu suspensa durante o ensaio e retomou do mesmo ponto. O teste também pressionou **E** no relé da Elétrica e registrou um único estalo, 1,483 s após o instante de entrada marcado no roteiro.

No teste `--smoke-hunt`, o primeiro roteiro ainda esperava a desistência rápida da dev.5. O teste revisado verificou busca temporária atrás da porta, ausência de contato e encerramento dentro do orçamento. O executável final passou em 2.055 quadros, com 14 apoios de mão e 12 de pé na campanha. O pico do exercício reto continuou em 4,95 m/s.

## Regressões no executável final

| Roteiro | Resultado | Cobertura |
|---|---|---|
| `--smoke-inteligencia` | PASS · 554 quadros | Relé físico, bot com desvio de 1,44 m, pausa, replay, obstáculos e exportação |
| `--smoke-hunt` | PASS · 2.055 quadros | Caçada física, porta, captura silenciosa, Despertar e isolamento do Lab |
| `--smoke-observer` | PASS · 2.880 quadros | Percepção, navegação, porto seguro e circulação da Presença física |
| `--smoke-motion` | PASS · 1.520 quadros | Importação do rig, clipes e 112 eventos de animação |
| `--smoke-sonoro` | PASS | Áudio espacial, dispositivo ativo, oclusão, fontes isoladas e som da Estação |
| `--smoke-test --check-audio` | PASS · 12.260 quadros | Vistoria da Bomba 02, mapa, colisão, efeitos de áudio e saída |
| `--smoke-reality` | PASS · 2.696 quadros | Continuidade, falsa realidade, recuperação e saída verdadeira |
| `--smoke-echoes --check-audio` | PASS · 2.735 quadros | OS 17, Câmera 02 e três emissores espaciais |
| `--smoke-collapse` | PASS · 8.548 quadros | Instabilidade, pausa, recuperação tardia e despedida |

A rota completa da vistoria mantém a representação legada no roteiro geral. As provas específicas da Presença física exercitam seleção, aproximação, contato, retomada e consequências na campanha. Sessões completas de jogabilidade com a representação nova ainda são parte da avaliação humana.

## Compilação e integridade

Rustfmt, Clippy com `-D warnings`, 95 testes do workspace, compilação com `dev-lab` e checagem sem a feature concluíram com sucesso.

| Arquivo | SHA-256 |
|---|---|
| ZIP experimental dev.6 | `bbfc353f5a6dc426911930ca7e36fd2445b6d7ba984ed0f8d00756877ddcaf1d` |
| Executável Windows x64 da dev.6 | `61ec6f08999eae4c2b5483155e42776a235fe5bda3996156bd0a4c47d2061f0b` |
| ZIP dev.5 preservado | `f23d35b3490283af15d971f081585345032cb108a2725c1ed30aaf93a867a2e7` |

O pacote contém o executável, o atalho experimental, as licenças e o guia de abertura. O binário foi conferido após extração isolada. O modelo, os clipes, o mapa, o núcleo da missão e o inicializador universal permanecem nos arquivos da base.

## Avaliação humana

Os ensaios usam sequências automatizadas e geometria controlada, incluindo uma execução gráfica na Estação. A taxa de contato nos exercícios de pista descreve apenas aqueles corredores. A frequência percebida de encontros, o conforto e o medo precisam da avaliação do diretor e dos testers.
