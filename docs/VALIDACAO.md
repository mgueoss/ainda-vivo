# Validação — 17/11/2026

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
