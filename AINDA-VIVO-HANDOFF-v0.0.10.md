# AINDA VIVO v0.0.10 — Ecos do Desaparecimento

Este documento descreve a lógica interna e revela desfechos. Para jogar, baixe o ZIP Windows x64, extraia e abra **AINDA-VIVO.exe**. [Release v0.0.10](https://github.com/mgueoss/ainda-vivo/releases/tag/v0.0.10) · [Validação e capturas](docs/VALIDACAO-v0.0.10.md)

## Base e experiência

A base é o commit `ca1704b`, identificado pela tag v0.0.9. O projeto continua em Rust, edição 2024, Bevy 0.18.1 e física cinemática própria a 60 Hz. O adaptador Windows escreve `PlayerCommand`; a câmera interpola o corpo. Reutilizamos a geometria e os colisores da Estação 07, o pós-processamento VHS, a lanterna, a síntese sonora no executável, o inicializador Win32 e o cache. A árvore de dependências permanece a mesma.

O operador recolhe o fusível, faz a leitura inicial da Bomba 02, rearma o quadro, volta à bomba e cruza a comporta após liberar a saída. Essa rota dá acesso aos desfechos independentemente da investigação da OS 17. A ficha deslocada, o livro de ponto e o gravador são pistas ambientais. O operador ganha outra interpretação do turno ao juntar as pistas.

## Propriedade de estado

| Estado | Onde vive | Quem o consome |
|---|---|---|
| Fase da vistoria e estado real da comporta | `Expedition::truth()` em `crates/ainda_vivo_core/src/reality.rs` e regras de `story.rs` | Ações autoritativas enquanto o participante está vivo. |
| Participação do operador, ponto de retorno e limites | `Observer`, identificado por `ObserverId` | Captura, Despertar, progressão percebida, término e retomada. |
| Cópia percebida da vistoria após captura | Variante privada `Participation::False`, exposta por `Expedition::view()` | Interações, portão, colisão do portão, objetivos, relógio e registros locais. |
| O que foi visto, lido ou ouvido no trajeto | `FieldMemory` e `ServiceTrail` em `crates/ainda_vivo_core/src/memory.rs` | Diretor, figura, passos, condições da OS 17 e frase de desfecho. |
| Objetos, iluminação, áudio e gravação | `src/scene/{illusion,presence,recording}.rs`, `src/atmosphere.rs` e `src/atmosphere/sound.rs` | Renderização e mistura estéreo no cliente local. |
| Temporização, seed e histórico recente | `src/atmosphere/director.rs` | Janelas concorrentes de som, ilusão e aparição. |
| Confirmação e mensagem de fim | `src/app.rs`, `src/ui.rs`, `src/desktop.rs` | O HUD de jogo continua usando a mesma projeção antes e depois da captura. |

A atualização separa a memória de cada operador da memória de curto prazo do diretor. Para LAN futura, associe comando, checkpoint e efeitos privados ao `ObserverId`, mantendo verdade, presença compartilhada e validação espacial no host. A visão atual da comporta é local e suficiente para o modo solo. Em rede, colisão percebida e autoridade do host precisarão de regras explícitas, bem como a remoção visual da pessoa capturada para os demais jogadores. Android ainda requer adaptação de entradas, ciclo de vida, renderização, áudio e orçamento de memória.

## Memória e mudança de objetos

`FieldMemory` tem capacidade fixa. Contém contadores saturados para 16 identificadores de setor, os quatro setores mais recentes, ações de consulta e uso, cinco tipos de objeto, o horário da última aparição percebida, um par de intervalos de passos e o estado da luminária. O registro de setor ignora quadros consecutivos dentro do mesmo ambiente. O primeiro olhar útil exige 0,30 s de atenção contínua em até 5,5 m. O vetor da câmera precisa alinhar com o objeto (produto escalar acima de 0,89) e o raio passa pelos colisores da cena. Tempo de observação interrompido é descartado. Cada memória guarda a variante efetivamente vista.

A troca visual usa o mesmo objeto de cena e ocorre quando ele sai do enquadramento (produto escalar abaixo de −0,12) e o diretor concede a janela. Há variantes para ficha, indicador de ramal, relógio de Controle, etiqueta de trabalhador e relé. Variantes tardias de ficha e relógio exigem distorção ativa, revisita e 14 s desde a troca. A mudança é cosmética. A ficha só vai para a Elétrica depois de ter sido realmente vista, com o local de origem a mais de 12 m e o destino fora de vista. Nenhuma entidade extra, porta ou colisor é criado para a ficha transferida.

O diretor mantém janelas esparsas, histórico de seis sinais, pressão que decai e pausas protegidas. A OS 17 pode orientar uma janela para áudio ou ilusão. Movimento brusco da câmera, um contato ativo, um momento autoral de silêncio ou uma mesa iluminada podem adiar a janela. Algumas janelas ficam vazias. A Presença conta um avistamento após 0,46 s útil em quadro. Revisitar a Triagem depois de confrontar a via assinada permite outra aparição em condições válidas de distância, oclusão e enquadramento. A figura ainda usa pontos autorais e pode ser evitada.

## Como investigar a OS 17

| Etapa privada | Condição e efeito |
|---|---|
| `Unknown` → `Briefed` | Leia com **E** a segunda via azul na mesa do Arquivo. A nota indica o relé junto ao quadro e o cancelamento às 02:52. |
| `Briefed` → `Footfalls` | Caminhe o suficiente para formar dois intervalos válidos (0,23–1,10 s). A partir da fase Elétrica e de 7 s após ler (5 s com distorção ativa), uma janela de som pode tocar três passos de cadência semelhante em um ponto afastado. A origem fica entre 3,35 e 10,5 m e fora do eixo de visão. |
| `Footfalls` → `Relay` | Observe a placa original de perto e vire o olhar. Depois de 9 s do eco (6,5 s com distorção), uma janela de ilusão pode trocar a placa por uma via assinada às 03:19. A ficha conhecida pode reaparecer abaixo do relé. |
| `Relay` → `Checked` | Use **E** na placa. A mensagem associa a assinatura de Borges à Câmera 02. A leitura física pelo raio funciona dentro do alcance normal. |
| `Checked` → `Reconciled` | Após 8 s e com o quadro rearmado, consulte o monitor no Controle. A câmera mostra a OS assinada às 03:17 e o texto informa que o relé seguia lacrado. A consulta anterior a esse intervalo ainda mostra o quadro original. |

O estágio registra a emissão que aconteceu no mundo percebido, mesmo se o jogador usar mudo. Isso preserva a descoberta por pistas visuais. O áudio usa o mesmo banco procedural e recebe atenuação por distância, panorâmica e abafamento por sólidos do mixer. Os três passos são um acontecimento único por cadeia, agendado com os dois intervalos lembrados. A mesa do Arquivo tem um interruptor funcional. Perto da mesa acesa, luz e ganho de ambiência convergem suavemente para uma experiência mais estável. Pausar suspende o relógio da expedição, as fontes e os ramais pendentes.

A câmera já existente aponta para a nave. Usa alvos RGBA de 384 × 216 e camadas de renderização. A primeira imagem mostra uma figura. O segundo alvo recebe um quadro com a via OS 17 por uma janela de cerca de 0,55 s. A câmera volta a ficar inativa e o monitor reutiliza o quadro como material. A duplicação é de um alvo de imagem da câmera, equivalente a cerca de 324 KiB adicionais em RGBA8 para a textura resolvida, sem contar buffers transitórios.

## Cronologia autoral de Borges

| Horário no papel | Evidência e intenção |
|---|---|
| 02:48 | O livro de ponto no Controle tem uma assinatura de saída. É o registro administrativo de referência para o turno 07. |
| 02:52 | O carimbo na segunda via cancela a inspeção do relé agendada para 03:19. A OS 17 é separada da atual Ordem 07 da Bomba 02. |
| 03:12 | A ocorrência do Arquivo relata pancadas, prancheta no piso e uma linha de saída em branco. O gravador registra dois operadores. A inconsistência com 02:48 já fazia parte do mistério da v0.0.9. |
| 03:17 | O quadro alternativo enquadra a via assinada antes de constar abertura do lacre e antes do horário indicado na placa. A mensagem de portaria pode repetir o horário. |
| 03:19 | A via reaparece concluída e com tinta borrando apesar do cancelamento. É uma contradição intencional da percepção. |

Preserve a ambiguidade sobre quando cada registro foi produzido e o que aconteceu aos trabalhadores. Os horários acima são evidências encontradas pelo jogador em uma planta industrial; nenhum texto confirma a origem da Presença.

## Captura, Despertar e desfechos

- O contato exige rearme real, tentativa elegível e uma figura percebida em um ponto de risco. O rastro dura até 24 s. Entrar a até 4,4 m, com desnível e linha de visão admissíveis, emite o aviso sonoro. Permanecer exposto por 2,15 s, de pé ou agachado com os pés no chão e velocidade horizontal abaixo de 1,55 m/s, pode capturar após pelo menos 2,4 s do aviso. A janela encerra após 6 s. Afaste-se ou volte a iluminar a direção do rastro para interromper a exposição.
- A captura copia apenas a estrutura de progresso para o visor. A transição mantém corpo, comando, orientação, lanterna, relógio, mensagem e fontes ativas. O diretor leva pelo menos 4 s para favorecer a distorção. Pistas compartilhadas pelos dois estados impedem que um evento isolado resolva a dúvida.
- **Despertar** fica oculto fora da pausa e sempre pede confirmação. Na falsa realidade, devolve uma vez ao posto da Elétrica e volta à fase real da missão. Preserva o que foi aprendido da OS 17, limpa o registro de variantes vistas e restitui a placa correspondente ao estágio já conhecido. Uma recuperação bem-sucedida impede nova captura naquela tentativa.
- Vivo, o mesmo comando encerra a tentativa. Se já houve rearme real e a pessoa ainda tem direito à retomada, a tela oferece **RETOMAR DO POSTO**. A nova tentativa guarda somente a fase alcançada com ações reais, mantém a posição do posto como ponto seguro e limpa memória, eventos, temporizadores e imagem consultada. Antes do rearme, ou depois de gastar esse direito, a tela oferece um novo início completo. A retomada curta continua sujeita à Presença. A sessão tem memória volátil.
- A saída real conclui e exibe uma variante de registro quando a cadeia foi confrontada. A saída percebida permanece jogável. A portaria confirma a baixa por 4 s, ou 5,5 s após confrontar a OS 17, e só depois mostra a contradição. O silêncio e a luz de alívio precedem a suspeita. Despertar continua disponível na saída percebida.

## Desenvolvimento e reprodução

```powershell
cargo fmt --all -- --check
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
cargo test --locked --offline -p ainda-vivo -p ainda_vivo_core
cargo test --locked --offline -p ainda-vivo-launcher
cargo build --locked --offline -p ainda-vivo
Start-Process $exe -Wait -PassThru -ArgumentList '--smoke-test --check-audio --capture-dir qa/rota --smoke-report qa/rota.txt'
Start-Process $exe -Wait -PassThru -ArgumentList '--smoke-reality --seed 20261118 --check-audio --smoke-report qa/realidade.txt'
Start-Process $exe -Wait -PassThru -ArgumentList '--smoke-echoes --seed 20261118 --check-audio --capture-dir qa/os17 --smoke-report qa/os17.txt'
```

Nos comandos acima, `$exe` aponta para a build do jogo e `qa` deve existir. Os testes Cargo em dois grupos reutilizam as combinações de dependências já compiladas para o jogo e o launcher. O smoke usa 60 Hz de tempo virtual e entradas controladas. O teste da OS 17 mede também intervalos reais de quadros em um trecho específico, usando `Instant`. **F3** mostra somente métricas de movimento e quadros. Para depuração com spoilers, execute com `--dev-reality --seed 20261118`. **F6** tenta capturar sob as restrições autoritativas, **F7** pede uma janela válida de ilusão e **F8** registra estado, seed, eventos, setores recentes e a fase da OS 17. As flags de smoke selecionam rotas automatizadas.

Use `scripts/distribution` para verificar o PE, empacotar o arquivo testado e utilizar a credencial Git já configurada. Os arquivos distribuídos têm verificação SHA-256 e validação pelo launcher. [Distribuição](docs/DISTRIBUICAO.md) · [Validação externa](docs/VALIDACAO-v0.0.10.md)

## Problemas conhecidos e continuidade

- A cadeia opcional depende de revisitas e da direção do olhar. Pode ficar incompleta numa passagem rápida. Para dosar a descoberta, observe diretores e jogadores em sessões livres e registre onde percebem o relé e os sons.
- O áudio é estéreo procedural com oclusão simplificada. A imagem da nave é uma captura autoral; não há gravação histórica contínua. O acabamento usa geometria e materiais procedurais. O balanço segue opcional e a luz, etiquetas e interações funcionam com VHS desligado.
- O teste de rendimento cobre um trecho em uma máquina. Falta testar sessões livres, conforto subjetivo, máquinas de entrada e Windows limpo com instalação efetiva do runtime. A validação automatizada cobre regras e transições.
- O jogo segue solo Windows x64, com uma vistoria e salvamento volátil. A próxima revisão deve avaliar ritmo e legibilidade com o diretor antes de expandir a campanha ou ajustar requisitos. Android, LAN, calibração automática e instalação modular permanecem fora desta atualização.

## Referências de direção

O ritmo usa a rotina da vistoria, oportunidades de interpretação e ameaças com espaço entre encontros. A gravação e a ficha dão consequências perceptíveis à exploração. A escolha de deixar janelas vazias reduz o aprendizado de padrões de repetição. [Pilares de SOMA, Frictional Games](https://frictionalgames.com/2013-12-the-five-foundational-design-pillars-of-soma/) · [Alien: Isolation e os problemas de repetição, Frictional Games](https://frictionalgames.com/2014-05-alien-isolation-and-the-two-hardest-problems-in-horror/) · [Gaps of the Imagination, Frictional Games](https://frictionalgames.com/2017-06-gaps-of-the-imagination/)
