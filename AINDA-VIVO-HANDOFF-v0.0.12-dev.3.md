# AINDA VIVO v0.0.12-dev.3 — O Observador

A base é a dev.2, commit `1a021e1`. A nova Presença é um experimento do Dev Lab e da campanha Windows. O Blender, o GLB, as texturas e o manifesto de clipes continuam os da dev.2. [Handoff da dev.2](AINDA-VIVO-HANDOFF-v0.0.12-dev.2.md) · [Dev Lab](docs/DEV-LAB.md) · [Validação](docs/VALIDACAO-v0.0.12-dev.3.md)

## Código e fronteiras

| Arquivo | Responsabilidade |
|---|---|
| `src/lab/observer/brain.rs` | Máquina de estados, visão, audição, memória, seleção de destinos e movimento. Recebe uma visão local, ruídos e acesso espacial. Marcadores para futuros pedidos de perseguição, contato e captura retornam indisponíveis. |
| `src/lab/observer/nav.rs` | A* em uma grade de 0,8 m. Arestas de quatro vizinhos são pré-calculadas contra a geometria expandida. Portais conectam os setores. As rotas consideram a porta e o corpo faz uma checagem contínua do segmento antes de avançar. |
| `src/lab/observer.rs` | Adaptador de campanha, eventos de passos e equipamentos, proteção de interações, escolha do posto inicial e ponte para a animação. |
| `src/lab/observer/test_lab.rs` | Corredor de ensaio, porta, controles da IA, estímulos e guias. |
| `src/lab/library.rs` | Importação, instâncias, mistura dos clipes existentes e pausa do grafo. |
| `src/lab/observer/qa.rs` | Roteiro nativo em Windows. Exercita a janela do diretor em campanha, os setores e a porta no Lab. |

A campanha experimental pede uma janela ao diretor quando há condições de manifestação. Fora das consultas sensoriais, a posição e o olhar do jogador são usados para colocar o corpo fora do quadro e autorizar sua retirada fora de vista. A orientação inicial é cardinal. Para decidir aonde ir, a criatura usa as referências do mapa, a última visão confirmada e a localização aproximada de ruídos. A representação anterior continua sendo a opção inicial. O F5 solicita a troca de representação. Com a física ativa, o diretor continua administrando seus demais eventos. O estado do observador fica separado da missão, da captura silenciosa original, da sanidade e da falsa realidade.

## Comportamento

O controlador reúne Ausente, Repouso, Exploração, Observação, Investigação, Espionagem, Acompanhamento, Aproximação, Intimidação e Retirada. O encerramento de um encontro depende de ficar fora de vista. A próxima oportunidade respeita um intervalo entre 24 e 49 segundos, somado às janelas do diretor. O estado guarda os quatro estados anteriores e últimos setores atingidos. A rodada de decisões leva cerca de 0,14 s. A simulação usa segundos e a avaliação visual roda independentemente. A proteção pode vir de leitura, interação, refúgio, pausa e estados específicos da sessão.

A visão usa alcance de 20 m e cone horizontal de 100° por padrão, com aproximadamente 0,25 s de exposição sem obstrução. Orientação da cabeça e altura da postura influenciam o teste. A última posição visual vive por 9 s. Ruídos lógicos têm atenuação por distância, perda pela geometria e até três conexões de setores. A referência aproximada usa 1,6 m quando a aproximação preserva o setor e dura 8 s.

A navegação usa as paredes, equipamentos, pisos e comporta da Estação. O interior rebaixado do tanque fica vedado ao controlador plano. A postura em pé usa envoltória de 1,00 × 2,24 × 1,00 m. A ruptura exige um espaço conservador antes de comandar o clipe e a postura baixa ocupa até 1,36 × 1,30 × 2,48 m. A IA usa a postura baixa para intimidar parada. O destino e o caminho ficam em cache até o movimento terminar, o acesso mudar ou o avanço falhar. Replanejamentos têm intervalo de 0,8 s.

## Trabalhar na etapa

```powershell
cargo build --locked --features dev-lab
./target/debug/ainda-vivo.exe --presence-physical --seed 20261118
./target/debug/ainda-vivo.exe --smoke-observer --seed 20261118 --capture-dir capturas --smoke-report observador.txt
```

O pacote Windows usa `scripts/presenca/empacotar-dev.ps1` e o mesmo conjunto de dependências da dev.2. Priorize nas próximas integrações a leitura de espiadas em ângulos reais, perfis diferentes de exposição e um envelope de navegação para movimentos quadrúpedes. Use as interfaces de intenção quando o diretor aprovar consequências para novos encontros.
