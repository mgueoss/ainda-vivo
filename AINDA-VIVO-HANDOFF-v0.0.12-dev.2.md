# AINDA VIVO v0.0.12-dev.2 — Movimentos Antinaturais

A etapa dá movimento à Presença no Dev Lab. A base funcional é a v0.0.12-dev.1, commit `8237849`. A v0.0.11, seu lançamento no GitHub e a campanha ficaram como referência. [Handoff da etapa anterior](AINDA-VIVO-HANDOFF-v0.0.12-dev.1.md) · [Uso do Lab](docs/DEV-LAB.md) · [Validação](docs/VALIDACAO-v0.0.12-dev.2.md)

## Ativos e direção de movimento

- Editável em `art/presenca/presenca.blend`. O GLB `assets/models/presenca.glb` contém 39 ações, com as sete anteriores nos mesmos índices. `assets/models/movimentos.json` vincula os 32 clipes novos aos estados, tempos, eventos, janelas de apoio, deslocamento assinado e postura. Coordenadas da fonte em metros, frente `−Y` no Blender e `+Z` no Bevy.
- As vigílias isolam cabeça, pescoço e respiração. O giro permite que a cabeça lidere a rotação do corpo. As caminhadas concentram o peso nos pés e mantêm um braço mais quieto que o outro. A inclinação extrema usa 44 graus divididos em duas articulações. O estúdio oferece um banco de referência para sentar e levantar.
- A ruptura baixa a cabeça e o quadril, flexiona a coluna, planta as mãos em momentos diferentes e reorganiza as pernas com duas passadas. A versão interrompida recua antes de firmar as mãos. A lenta tem 4,70 s, a rápida cerca de 1,13 s e a incompleta 2,60 s. A mesma superfície é deformada o tempo inteiro.
- Na corrida, coluna e ombros alternam compressão e extensão. Mãos e pés têm fases assimétricas e alvo explícito durante o apoio. As amplitudes incluem respiração e postura de preparação, arrancada, marcha moderada, corrida intensa, inclinação em curva, desaceleração, parada brusca, retirada e retorno em pé. A vigília quadrúpede e o aceno usam pausas próprias.
- A fonte conserva a geometria, as posições de repouso do rig e seus pesos. As correções de ombro, punho, dedos e joelho foram feitas nas poses e nos alvos dos membros. O Blender abre em posição neutra. Selecione **Presença · rig** e use o **Editor de Ações**, ou ative uma pista do editor de animação não linear por vez.

## Arquitetura

| Local | Responsabilidade |
|---|---|
| `scripts/presenca/movimentos.py` | Gera poses a 30 Hz, resolve membros com duas articulações e grava quaterniões contínuos. Exporta ações e associa seus tempos reais do GLB aos metadados. |
| `scripts/presenca/auditar_movimentos.py` | Mede a superfície deformada e os apoios amostrados a 60 Hz. Compara rig, malha e pesos com a dev.1. |
| `src/lab/motion.rs` | Estados semânticos, relógio, integral de velocidade, eventos, envelopes por postura e roteiro por distância. |
| `src/lab/library.rs` | Associa clipe a nó Bevy, avalia o relógio de cada instância, mistura nós e atualiza transformações e guias. |
| `src/lab.rs`, `panel.rs` e `room.rs` | Comandos, persistência da cena, câmera, linha do tempo, banco e pista no ambiente de desenvolvimento. |
| `src/lab/qa.rs` e `motion_qa.rs` | Testes nativos da sala, das fronteiras da campanha e dos clipes. Captura de PNGs pelo renderizador. |

`State::preferred_clip()` mapeia Idle, Watch, Walk, Peek, Retreat, Rupture, Quadruped Idle, Quadruped Sprint, Stop, Contact e Farewell a um clipe de referência. Rotinas podem escolher uma variante pelo número. O controlador usa o tempo do clipe e o Bevy avalia os ossos. Uma mudança durante a reprodução guarda os pesos atuais, congela as poses de origem e mistura por 80–200 ms. Uma nova interrupção retoma a partir dos pesos correntes. O avanço manual vai diretamente à amostra escolhida. O controlador desativa a avaliação do grafo enquanto a instância conserva uma pose.

`MotionEvent` contém a entidade da instância e o tipo. Os tipos incluem os quatro apoios de mãos e pés, início da ruptura, início da corrida, impacto, contato e fim. A reprodução despacha os marcadores cruzados em ordem de tempo. Voltar, avançar ou arrastar a linha do tempo só altera a pose. A campanha fica suspensa enquanto o laboratório está aberto. O modo de prévia autorizado em campanha mostra a malha estática.

`Posture::dimensions()` retorna dimensões experimentais X/Y/Z em metros. Em pé, 0,82 × 2,24 × 0,78. Em apoio quadrúpede, 1,18 × 1,30 × 2,43. As poses de ruptura e retomada interpolam entre os envelopes. O deslocamento manual usa uma checagem simples com os sólidos do Lab. A pista e a aproximação usam trajetórias predefinidas. A aproximação aponta para um posto fixo de observação e o controlador preserva a distância mínima da marca. Colisão autoritativa e navegação da Presença ficam para a integração futura.

## Reproduzir a geração e o teste

```powershell
git archive --format=zip --output=base-dev1.zip 8237849 art/presenca/presenca.blend
Expand-Archive -LiteralPath base-dev1.zip -DestinationPath base-dev1
blender --background --factory-startup --python-exit-code 1 --python scripts/presenca/movimentos.py -- base-dev1/art/presenca/presenca.blend arte-dev2
blender --background --factory-startup --python-exit-code 1 --python scripts/presenca/auditar_movimentos.py -- arte-dev2 base-dev1/art/presenca/presenca.blend auditoria.json
cargo build --locked --features dev-lab
./target/debug/ainda-vivo.exe --smoke-motion --lab-config lab-qa.json --capture-dir capturas --smoke-report movimentos.txt
```

O gerador usa Blender 4.5.16 LTS. Copie o `.blend` de saída para `art/presenca` e os outros dois arquivos para `assets/models` antes de recompilar. Faça a montagem Windows com `scripts/presenca/empacotar-dev.ps1` e os avisos de licença da feature `dev-lab`. Para registrar a aproximação, acrescente `--lab-record-motion` ao teste. O script `gravar_demonstracao.py` codifica os PNGs em `capturas/video`.

## Continuidade

A produção da próxima etapa pode partir dos estados e eventos depois da aprovação do diretor. Nas curvas e misturas, os alvos dos membros continuam os das ações autoradas em espaço local. Há margem para refinar a inclinação do corpo para cada lado e aplicar compensação de contato no espaço real quando houver navegação. A demonstração deve servir para decidir a leitura da ruptura, distância de aproximação e peso da corrida. Os testes automatizados verificam a execução e os apoios. A aprovação artística e de conforto depende dos testes humanos.
