# Validação — 18/11/2026

A validação da versão mais recente está em [Validação v0.0.10](VALIDACAO-v0.0.10.md), com as rotas de regressão e da OS 17, áudio, capturas e integridade da distribuição.

## v0.0.8 — Além da Escuridão

Rust/Cargo 1.97.1, Windows x64 MSVC, Bevy 0.18.1 e NVIDIA GeForce GTX 1660 SUPER com driver 610.88 via Vulkan. A validação reutilizou o cache na unidade D: e compilou em modo offline. O executável distribuído usa o perfil de desenvolvimento otimizado do projeto, com símbolos de depuração desativados.

| Verificação | Resultado |
|---|---|
| `cargo fmt --all -- --check` | Aprovado |
| `cargo clippy --locked --offline --workspace --all-targets -j 2 -- -D warnings` | Aprovado |
| `cargo test --locked --offline --workspace -j 2` | 29 aprovados, 0 falhas. São 16 no aplicativo e 13 no núcleo. |
| `cargo build --locked --offline -j 2` | Aprovado. O build final usou 45,41 s com o cache disponível. |
| Smoke nativo com `--check-audio` e capturas | PASS em 12.260 quadros de teste, 21 capturas, 0 falhas. |

- **Missão.** O script caminhou pelos trechos da ida à Bomba 02, da Elétrica, da volta à bomba e da Decantação. Conferiu pressão e trava, pausa, colisão, agachamento, interação, lanterna e reinício. A auditoria percorreu 15 setores, 22 portais bidirecionais e dez linhas de visão das aparições.
- **Presença e diretor.** Quatro pontos ficaram enquadrados na rota principal, incluindo a travessia da passarela registrada com VHS suave. Um enquadramento dedicado conferiu permanência sob o olhar direto e desaparecimento após virar. A primeira tentativa completa, incluindo as etapas focadas depois da rota, registrou cinco aparições, dois incidentes sonoros, uma ilusão, cinco janelas expiradas sem evento, nove adiamentos e cinco revisitas. Os quatro modos ocorreram.
- **Ilusões e registros.** A ficha mudou de pose depois de o jogador virar e voltou à pose original no reinício. O indicador do Controle alterou material fora do enquadramento. O teste leu o livro de ponto e a ocorrência do Arquivo via interação real, e recuperou a gravação da nave.
- **Áudio.** O dispositivo padrão abriu e as vozes avançaram durante as cenas. O teste confirmou três tipos distintos de ruído incidental, o retorno da oficina e os dois sons autorais da Bomba 02 e Decantação. Volume percebido e identificação da origem ainda dependem de escuta humana.

O smoke usa semente fixa, simula a entrada, reposiciona o jogador em pontos específicos e controla o tempo de simulação. Os valores de FPS das capturas não são medições de desempenho. Nenhum FPS mínimo ou ganho percentual de desempenho foi aferido. Sensibilidade do mouse, conforto e impacto do terror ainda precisam de sessão livre com o diretor.

[Relatório bruto](smoke-report-v0.0.8.txt) · [Travessia na passarela](imagens/v0.0.8/passarela-vulto.png) · [Ficha original](imagens/v0.0.8/ficha-antes.png) · [Ficha deslocada](imagens/v0.0.8/ficha-depois.png) · [Indicador original](imagens/v0.0.8/ramal-antes.png) · [Indicador aceso](imagens/v0.0.8/ramal-depois.png) · [Nave sem VHS](imagens/v0.0.8/nave-alcance.png)

## v0.0.7 — Vistoria da Bomba 02

- Compilação offline, rustfmt e Clippy com `-D warnings` aprovados. Os 25 testes passaram.
- O teste gráfico de 5.160 quadros percorreu a ida à Bomba 02, o quadro da Elétrica, a volta à bomba e a saída pela Decantação. A comporta ficou trancada durante o rearme e abriu depois da segunda leitura. O teste também conferiu os 15 setores, 22 passagens e a posição do ponteiro nas duas leituras.
- Dez linhas de visão foram verificadas. Cinco aparições ficaram enquadradas durante a expedição automatizada. As capturas mostram a travessia da passarela com VHS suave, o rastro na água e o quadro arquivado no monitor. O reinício limpou os estados da vistoria e do gravador.
- A execução gráfica rodou na GTX 1660 Super via Vulkan. Foi confirmada a reprodução das pancadas da bomba, do arrasto no corrimão e de quatro tipos de ruído de fundo. [Relatório da execução](smoke-report-v0.0.7.txt) · [Capturas e referências](PRESENCA.md)

## v0.0.6 — Visibilidade, aparições e detalhes da estação

- Compilação offline e Clippy com `-D warnings` aprovados. Os 22 testes do projeto passaram.
- O teste gráfico completou 4.000 quadros na GTX 1660 Super via Vulkan. Conferiu todas as 22 passagens, seis linhas de visão da Presença, uma aparição durante o percurso, o tempo de leitura sob o olhar direto, o movimento dos exaustores e das peças penduradas, a pausa e a reação do quadro geral à restauração da energia.
- Nas capturas do mesmo enquadramento com VHS suave, o vulto aparece sobre o fundo iluminado e desaparece após a espera. Na nave, o feixe torna o piso próximo legível enquanto o fundo some. Os quadros do exaustor mostram as pás em duas posições. [Relatório da execução](smoke-report-v0.0.6.txt) · [Capturas](PRESENCA.md)

## v0.0.5 — A Presença, lanterna e áudio

- Compilação offline, rustfmt e Clippy com `-D warnings` aprovados. Os 21 testes do projeto passaram.
- O percurso gráfico de 3.800 quadros passou no Windows com a GTX 1660 Super via Vulkan. O teste concluiu a expedição e conferiu as 22 passagens do mapa e as linhas de visão dos cinco pontos de aparição.
- A Presença apareceu durante o percurso. Uma segunda tentativa conferiu que o vulto surge à distância, se dissolve sob o olhar e deixa o mesmo cenário à vista. O teste comparou a lanterna ligada e desligada no mesmo enquadramento.
- A reprodução de três tipos distintos de ruídos ocasionais foi confirmada durante a execução. O teste também conferiu a pausa do áudio e o retorno da oficina. [Relatório da execução](smoke-report-v0.0.5.txt) · [Capturas e decisões](PRESENCA.md)

## Ambiente verificado

- Windows x64, alvo `x86_64-pc-windows-msvc`.
- `rustc 1.97.1` e `cargo 1.97.1`.
- Visual Studio 2022 Community, MSVC 14.44.35207; Rust, Cargo, rustfmt e Clippy já instalados.
- Bevy 0.18.1 fixado no manifesto; dependências resolvidas e registradas em `Cargo.lock`.
- Projeto criado em pasta nova, sem substituir um projeto existente. Cache e temporários dedicados no D:.

## Testes do núcleo

`cargo test -p ainda_vivo_core --locked --offline`: **12 passaram, 0 falharam**.

Cobertura de comportamento: varredura em parede fina a alta velocidade; deslizar na diagonal; parada em canto; caminhar paralelo ao piso; recuar da parede; colisão de teto; alcance/oclusão básica de raio; gravidade e estabilidade no chão; impedir levantar sob teto baixo; normalizar velocidade diagonal; pular e aterrissar; exigir fusível para abrir a saída e limpar progresso no reinício.

O script PowerShell foi analisado pelo parser sem erros de sintaxe. A formatação Rust foi conferida.

## Compilação e execução gráfica

- `cargo build --locked --offline -j 4`: **aprovado**, primeira compilação em 20 min 47 s. Nova chamada com cache: cerca de 2,4 s.
- Executável Windows gerado e aberto com sucesso.
- GPU detectada pelo renderizador: **NVIDIA GeForce GTX 1660 SUPER**, driver 610.88, backend Vulkan.
- Mapa procedural inicializado com 19 colisores.
- Teste gráfico de integração: **PASS**, 400 quadros, nenhuma falha. Verificou movimentação, chão, parede, pausa, agachamento, mirar + E no fusível, oclusão pela parede, mirar + E no quadro, passagem pela porta, conclusão e reinício.

A primeira revisão com Clippy foi interrompida por `STATUS_ACCESS_VIOLATION` dentro do compilador ao verificar `ttf-parser`. Repetida com um processo por vez, terminou normalmente. Os avisos de estilo do código do projeto foram corrigidos. **Clippy com `-D warnings` e rustfmt aprovados.** Os 12 testes foram repetidos e aprovados após esses ajustes. Não foi necessário alterar dependências.

A ferramenta externa de captura de janela perdeu a sessão duas vezes (`unknown or expired node_repl continuation`). A inspeção usa capturas do próprio renderizador. A primeira inspeção revelou glifos ausentes na fonte padrão; a interface foi corrigida para ler uma fonte já instalada no Windows, sem distribuí-la.

Menu, corredor e pausa foram conferidos visualmente em 1280 × 720 após a correção dos acentos. O teste de 400 quadros foi repetido na versão final e passou, incluindo as três capturas. O executável entregue em `windows/ainda-vivo.exe` é uma cópia com SHA-256 conferido do executável validado; o hash está em `windows/SHA256SUMS.txt`. Tamanho: 67.532.800 bytes, cerca de 64,4 MiB.

O teste automatizado reposiciona o jogador entre alguns pontos para verificar interação e oclusão; o trecho de saída é percorrido pela física. Isso não mede a duração de uma partida nem substitui explorar todo o mapa manualmente. A avaliação manual de mouse, conforto e sensibilidade pelo diretor continua necessária.

Capturas: [menu](imagens/menu.png), [corredor](imagens/corredor.png), [pausa](imagens/pausa.png). O FPS mostrado nessas capturas pertence ao tempo controlado do teste, **não é uma medição de desempenho**. Relatório reproduzível: [smoke-report.txt](smoke-report.txt).

## Limites da evidência

Testes automatizados não substituem a aprovação de conforto pelo diretor nem uma sessão prolongada no hardware de referência. O teste gráfico usa tempo de simulação controlado para reproduzir a sequência; seu FPS não é um benchmark. Android e LAN não foram implementados ou testados.
