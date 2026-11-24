# Validação da Etapa 1 · v0.0.12-dev.1

## Malha, rig e exportação

O Blender 4.5.16 LTS gerou e reabriu o arquivo editável. A malha fonte tem **18.800 triângulos, 9.400 vértices, um componente conectado e zero arestas não manifold**. A altura em repouso é **2,1961 m**. Todos os vértices têm pesos normalizados, com até quatro influências. Os ombros, a nuca e os joelhos foram suavizados depois da inspeção das primeiras revisões, que mostrou transições angulosas nas articulações.

O GLB exportado tem **18.800 triângulos e 12.185 vértices de renderização**, incluindo a separação das emendas UV. Ele ocupa 2.306.904 bytes. A reimportação no Blender confirmou uma malha deformada por um esqueleto de 52 ossos, um material, três texturas e sete clipes. Há cinco poses distintas, uma entrada neutra do rig e um aceno de demonstração.

| Textura | Resolução | Uso |
|---|---:|---|
| Cor | 1024 × 1024 | Pele, tecido, órbitas e vestígio do colar |
| Normal | 1024 × 1024 | Relevo discreto em espaço tangente |
| Rugosidade | 512 × 512 | Rugosidade no canal G e metalicidade zero em B |

### Apoio quadrúpede

Na malha deformada, a comparação dos pontos inferiores dominados pelos ossos de pés e pontas de dedos encontrou 1,8 e 2,2 mm para os pés e 7,7 e 8,1 mm para as mãos acima do plano Z=0. Pontos isolados da silhueta chegam a −6 mm. Essa é uma pose com contato aproximado. Antes de animar uma corrida completa, os contatos de mão e pé e a transição a partir da postura em pé precisam de validação ao longo de um ciclo.

| Pose | Alongamento de aresta no percentil 99 |
|---|---:|
| Imóvel | 1,00× |
| Cabeça inclinada | 1,04× |
| Espiando | 1,24× |
| Apoio quadrúpede | 1,85× |
| Despedida | 1,18× |

A medição usa arestas com mais de 2 mm em repouso. No quadrúpede, o pico local de 4,44× fica na transição de axila. A pose serve para avaliar a silhueta e os apoios. Os pesos locais de axila são o ponto a refinar quando a locomoção for testada.

## Testes do executável

Windows x64, GTX 1660 SUPER, Vulkan e driver 610.88. Rustfmt, Clippy com `-D warnings`, a compilação offline e a verificação sem a feature `dev-lab` passaram. A suíte Rust passou **63 testes**. São 27 do jogo, 33 do núcleo e 3 do inicializador universal.

```powershell
cargo fmt --all -- --check
cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings
cargo test --profile dev --locked --offline -p ainda-vivo -p ainda_vivo_core --features dev-lab
cargo test --profile dev --locked --offline -p ainda-vivo-launcher
./ainda-vivo.exe --smoke-lab --lab-stress --lab-config qa-dev-lab.json --capture-dir capturas --smoke-report qa-dev-lab.txt
```

Use um arquivo de preferências de QA novo para iniciar o roteiro com a cena padrão.

O roteiro nativo `--smoke-lab --lab-stress` passou em **4.101 quadros**. Ele conferiu importação do GLB, skin, escala, materiais, biblioteca, adição e remoção de objetos, colisores, câmera em primeira pessoa, câmera livre, salvamento e releitura das preferências, clipes, pausa, busca na linha do tempo e a pose quadrúpede no esqueleto em execução. A campanha entrou com colapso ativo. O roteiro confirmou a preservação da posição, da orientação, da fase da vistoria, do diretor e do tempo restante da crise, além da retomada do cronômetro na saída. [Saída do teste nativo](validacao/v0.0.12-dev.1/dev-lab.txt) · [Testes Rust](validacao/v0.0.12-dev.1/rust.txt)

Foram seis ciclos de entrada e saída no roteiro base e mais 32 alternâncias com a campanha pausada. Após as saídas da sequência em campanha, as quatro amostras repetiram **3.500 entidades, 237 malhas, 174 materiais, 102 imagens, zero câmeras do Lab ativas e zero instâncias selecionáveis**. Em pausa, as 32 amostras também repetiram a mesma contagem, com 103 imagens. A sala e a interface inativas têm uma única instância para reuso.

Na parte final do teste prolongado, entre 14 e 26,1 segundos desde o início do monitoramento, os **bytes privados do processo** ficaram entre 1.034,85 e 1.036,00 MiB. O **conjunto residente** ficou entre 475,44 e 477,53 MiB. As medições são do processo no Windows durante esse roteiro. VRAM, outros drivers e sessões de duração prolongada exigem medições próprias. [Amostras de memória](validacao/v0.0.12-dev.1/processo.csv)

O enquadramento frontal em estúdio teve mediana de **6,06 ms** em 25 intervalos de tempo de parede. A imagem de campanha mostra o HUD da simulação de física com passo de 1/60 s. Os valores de desempenho desta seção usam relógio de parede.

### Vistoria e comparação com a v0.0.11

A rota de realidade rodou com seed 20261118, áudio silenciado e sem capturas, na ordem v0.0.11, experimental, experimental e v0.0.11. Os tempos de parede incluem inicialização e encerramento. Cada repetição completou 2.696 quadros.

| Build | Execuções | Média |
|---|---|---:|
| v0.0.11 publicada | 18,102 s e 17,397 s | 17,750 s |
| v0.0.12-dev.1 com o Lab desabilitado | 17,530 s e 17,483 s | 17,507 s |

A comparação é um recorte automatizado no mesmo PC. A diferença entre médias foi menor que a variação entre as duas execuções da v0.0.11. [Tempos registrados](validacao/v0.0.12-dev.1/tempos.json)

- **Vistoria da Bomba 02** passou em 12.260 quadros. [Saída do teste](validacao/v0.0.12-dev.1/vistoria-dev.txt)
- **Realidade, captura e Despertar** passaram em 2.696 quadros. [Saída do teste](validacao/v0.0.12-dev.1/realidade-dev-a.txt)
- **OS 17 e Câmera 02** passaram em 2.735 quadros. [Saída do teste](validacao/v0.0.12-dev.1/ecos-dev.txt)
- **Colapso, recuperação e despedida** passaram em 8.545 quadros com o dispositivo de áudio dispensado pelo modo de teste. [Saída do teste](validacao/v0.0.12-dev.1/colapso-dev.txt)

### Pacote experimental

O ZIP foi extraído e o SHA-256 do executável conferido. **ABRIR DEV LAB.cmd** abriu a janela com a versão correta. O binário extraído passou mais **1.492 quadros**, importando o GLB do disco e executando a ação **Recarregar**. O roteiro confirmou que o catálogo recebeu a nova revisão do recurso. A versão publicada da v0.0.11 mantém o SHA-256 original do pacote e a tag aponta para `6afe2a5`. [Teste da cópia extraída](validacao/v0.0.12-dev.1/glb-em-disco.txt) · [Abertura pelo comando](validacao/v0.0.12-dev.1/atalho.txt)

| Arquivo | Bytes | SHA-256 |
|---|---:|---|
| ZIP experimental Windows x64 | 29.645.646 | `88308090f2cb21f5b5d37613567f142d35497831304cf5626d988bb83002b5e4` |
| Executável do ZIP | 83.390.976 | `6103114bd2c6e4aa367525e079ddbf731e7f754060f821f9da5c564ad169865e` |
| `presenca.glb` | 2.306.904 | `9ae7ee19f2557715667ebedc9a73a764f383dc80c1977bcd960a9a8ed42048f1` |

## Capturas do Bevy

Todas as capturas saíram da janela do jogo em 1280 × 720.

| Inspeção | Capturas |
|---|---|
| Anatomia | [Frente](imagens/v0.0.12-dev.1/01-frente.png) · [Perfil](imagens/v0.0.12-dev.1/02-perfil.png) · [Costas](imagens/v0.0.12-dev.1/03-costas.png) · [Rosto](imagens/v0.0.12-dev.1/04-rosto.png) · [Mão](imagens/v0.0.12-dev.1/05-mao.png) |
| Poses | [Cabeça inclinada](imagens/v0.0.12-dev.1/06-cabeca-inclinada.png) · [Espiada](imagens/v0.0.12-dev.1/07-espiada.png) · [Quadrúpede](imagens/v0.0.12-dev.1/08-quadrupede.png) · [Despedida](imagens/v0.0.12-dev.1/09-despedida.png) |
| Depuração | [Esqueleto](imagens/v0.0.12-dev.1/10-esqueleto.png) · [Wireframe](imagens/v0.0.12-dev.1/11-wireframe.png) · [Referência humana e figura antiga](imagens/v0.0.12-dev.1/12-comparacao-escala.png) · [Interface](imagens/v0.0.12-dev.1/16-interface-dev-lab.png) |
| Iluminação e jogo | [Luz Estação 07](imagens/v0.0.12-dev.1/13-luz-estacao.png) · [Emergência](imagens/v0.0.12-dev.1/14-emergencia.png) · [Só lanterna](imagens/v0.0.12-dev.1/15-lanterna.png) · [Prévia no corredor inicial da campanha](imagens/v0.0.12-dev.1/17-estacao-07-campanha.png) |
