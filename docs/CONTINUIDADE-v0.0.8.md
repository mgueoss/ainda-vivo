# AINDA VIVO v0.0.8 — documentação de continuidade

**Além da Escuridão · Windows x64 · Estação de Bombeamento 07**

Este documento descreve a base para continuar a programação e contém spoilers. O usuário é o diretor criativo. A tag local v0.0.7 fixa o commit 6065b69, com o código da versão anterior. O ZIP e o executável da v0.0.7 foram preservados. Consulte [arquitetura inicial](ARQUITETURA.md), [planta da estação](ESTACAO-07.md), [histórico de ambientação](PRESENCA.md) e [changelog](../CHANGELOG.md).

## Estado jogável

A expedição segue as fases Fuse → Baseline → Electrical → Return → Exit. O jogador recolhe o fusível na oficina, registra a pressão de repouso na Bomba 02, rearma o quadro geral, retorna para ler o mesmo manômetro e sai pela Decantação. A comporta fica trancada entre as duas leituras. As 22 passagens ligam 15 setores e permitem explorar rotas laterais. A planta em src/scene/station/plan.rs orienta a montagem, os sólidos e a auditoria de navegação. Coordenadas em metros, Y para cima e frente inicial em −Z.

O Arquivo guarda a ocorrência das três pancadas registrada por Borges às 03:12. O Controle tem um livro em que ele deu baixa às 02:48. O gravador atribui dois operadores à nave. O manômetro sobe no retorno e o lacre permanece intacto. Ler os documentos é opcional. A origem da Presença permanece em aberto.

A câmera fixa da nave renderiza uma textura em camada 3 com câmera, iluminação e silhueta próprias. Captura durante 0,55 s após entrar em Electrical e congela a textura pelo restante da tentativa. O terminal libera a consulta depois do rearme. A gravação é uma composição narrativa projetada para essa câmera.

O jogo usa WASD e mouse, Shift para correr, Ctrl/C para agachar, Espaço para saltar, E para interagir, F para a lanterna, Esc para pausa, Enter para iniciar ou continuar, F3 para diagnóstico e F4 para o VHS. Os padrões são FOV vertical de 65°, VHS suave, horizonte estabilizado com balanço desligado e VSync ativado. Após a compilação, mapa, texturas, máscaras, placas e sons são gerados localmente e funcionam offline.

## Organização do código

| Arquivo | Responsabilidade |
|---|---|
| Cargo.toml, Cargo.lock, .cargo/config.toml | Aplicativo v0.0.8, Bevy fixado em 0.18.1 e configuração do build. O crate ainda_vivo_core tem versão própria 0.0.1. |
| crates/ainda_vivo_core/src/story.rs | Phase, Action, Progress, objetivos, mensagens, travas e registro da consulta à câmera. |
| crates/ainda_vivo_core/src/{movement,collision}.rs | PlayerCommand, PlayerBody, AABB, varredura do volume, deslizamento, gravidade, chão, corrida, salto e agachamento. |
| src/app.rs | Plugins, etapas do Bevy, Session, Comfort, LocalPlayer, CollisionWorld e reinício. |
| src/desktop.rs, src/interaction.rs | Teclas e mouse, cursor, câmera, atualização do corpo a 60 Hz e interação a 2,4 m com tolerância de 3,5 cm para superfícies de apoio. |
| src/scene.rs, src/scene/station/ | Mapa, paredes, portas, máquinas, placas, iluminação, mobiliário, mecanismos e auditoria dos portais. |
| src/scene/{presence,illusion,recording,water}.rs | Figura, ilusões visuais, câmera da nave e rastro na passarela. |
| src/scene/surfaces.rs, src/vhs.rs, src/vhs.wgsl | Texturas, UV em metros, placas procedurais e pós-processamento do visor. |
| src/atmosphere.rs, src/atmosphere/episode.rs | Retorno da oficina, iluminação local e sequências autorais da Bomba 02 e Decantação. |
| src/atmosphere/director.rs, src/atmosphere/sound.rs, src/atmosphere/sound/ | Orquestração, síntese, posições incidentais, mistura e estéreo. |
| src/ui.rs, src/preferences.rs, src/diagnostics.rs | Menu, HUD, pausa, preferências locais e diagnóstico de quadro. |
| src/smoke.rs, scripts/dev.ps1, JOGAR.cmd | Teste integrado, automação para desenvolvimento e inicializador Windows. |

FixedUpdate aplica a entrada ao corpo, avança o relógio ativo e avalia a conclusão. Update ordena a entrada, a intervenção do smoke quando ativo, a câmera, a interação, a sincronização da estação, a atmosfera autoral, a água, a gravação, os mecanismos, o diretor, a Presença, as ilusões, o áudio e a interface. O diretor usa a posição física e a orientação escrita pela entrada. Enquadramento e interação usam a câmera interpolada. A lanterna compartilha sua posição e rotação.

Session::attempt identifica o reinício. Progresso, figura, agendas, ilusões e gravação se reiniciam por tentativa. A pausa para o relógio e os emissores de áudio. A física reaproveita duas listas de AABB, com e sem o sólido da comporta, reconstruídas quando CollisionWorld muda. A interação itera os sólidos sem copiar o vetor. O controle de aceleração, a varredura física e o agachamento continuam no núcleo existente.

## Diretor de terror

Director vive como Resource local no Bevy. Uma máscara de 16 bits distingue setores novos e revisitas. A cada 0,34 s, o diretor avalia velocidade horizontal e mudança na orientação do olhar. Uma rotação brusca, ausência de setor válido ou intervalo autoral protegido pode adiar uma janela. Hush suaviza a intensidade ambiente na suspeita e na tensão.

Uma janela de 3,5 a 6,2 s prioriza presença, som ou ilusão. Após 1,4 s outros candidatos visuais podem obter autorização. O sorteio pondera fase, velocidade e revisita. Depois de 14 s, parte das janelas é deliberadamente vazia. Uma janela também pode expirar sem candidato compatível. O primeiro consumidor validado chama claim, fecha a janela e registra o evento. Um intervalo adicional limita a repetição da última categoria. A atividade do vulto e os silêncios autorais da oficina, Bomba 02 e corrimão protegem sequências existentes. Os quatro modos são Calm, Suspicion, Tension e Relief.

A semente normal varia por tentativa e inicialização local. O smoke usa semente fixa. O relatório registra eventos por categoria, janelas vazias, adiamentos, revisitas e a máscara dos modos visitados. A execução depende de regras e de um gerador pseudoaleatório pequeno em Rust. Para ajustes de cadência, comece por Director::step e pelos cooldowns em allows e claim.

## A Presença

A tabela SITES contém dez marcas, testemunhas usadas pela auditoria, fases elegíveis e o movimento associado. O alvo inicial exige distância de 8,4 a 14,3 m, enquadramento e raio livre. O plano de 1,56 × 2,48 m usa máscara procedural irregular de 128 × 224. Ele é orientado com a câmera no começo da aparição. Aproximar-se a cerca de 6,4 m termina a aparição. Tempo e distância modulam a transparência.

Still mantém a pose na Triagem e no vão oeste da nave. Watch move discretamente a forma em Carga. Lean revela parte da figura nas máquinas e nos acessos. Cross atravessa o fundo da passarela. São necessários 0,46 s acumulados em quadro e alfa suficiente para registrar que o ponto foi observado. Depois disso, ao ficar 0,37 s fora de quadro, poses fixas e espiadas podem se retirar sem uma dissolução visível. A travessia preserva o curso do movimento. Um ponto ainda sem vislumbre suficiente pode tentar novamente após o intervalo. Uma revisita pode habilitar uma segunda aparição em uma marca já observada.

Para adicionar um ponto, escolha um fundo que permita perceber o contorno e um enquadramento alcançável. Inclua marca, testemunha, intervalo de fases e movimento em SITES. A auditoria verifica geometria. Confira a silhueta também com a iluminação e a trajetória reais.

## Ilusões e continuidade da morte silenciosa

Impression conserva pose e material originais e alterados. A ficha vermelha na Triagem muda 26 cm no eixo X e gira. A lente do ramal isolado no Controle pode acender a partir da fase Return. Cada objeto exige proximidade de 5,5 m, observação inicial com raio livre e que o jogador vire o olhar antes de o diretor autorizar a troca. A mudança dura até reiniciar. Os adereços são visuais e ficam fora da colisão e do conjunto de interações obrigatórias. Sons de passos se originam de pontos fora do enquadramento aproximado.

As ilusões atuais pertencem à única câmera local. Para a morte silenciosa futura, crie primeiro estado de jogabilidade explícito e identificador de observador. A decisão de participação cabe ao núcleo ou à sessão autoritativa. Apresentação e áudio podem reagir a esse estado por observador. A futura LAN precisará de um caminho de renderização local por visor ou cliente para que as transformações possam divergir.

## Áudio, imagem e custo

- Amostras mono f32 a 24 kHz são geradas no startup. Voice entrega estéreo. Laços têm crossfade. Amostras são compartilhadas por Arc. Cada família incidental tem três variantes de timbre e intervalo. O agendador evita repetir a família imediatamente e usa pontos no mapa a 4–15 m. Passos humanos exigem um ponto fora do campo de visão aproximado.
- A mistura aplica queda com distância e pan de potência constante. O raio de oclusão reduz o ganho e aplica filtro quando encontra um obstáculo. Ganhos e abafamento são transmitidos ao decoder em um AtomicU32 e suavizados. A localização é estéreo e a reverberação é sintetizada por amostra.
- O Bevy usa PBR, luzes locais e sombra na lanterna. A luz alcança 15,1 m. A névoa escura aumenta entre 5,2 e 18,6 m. O visor usa MSAA 4×. A opção de balanço desloca a câmera em até 5 mm. Desligar o VHS pula o passe gráfico do visor principal. Os materiais geram escorrimentos na tinta, desgaste no concreto e textura independente do VHS.
- O diretor usa listas fixas de candidatos e verifica contexto por amostragem. O cache elimina a reconstrução da lista de sólidos a cada passo fixo. Tempo de quadro e consumo em jogo comum ainda precisam de benchmark comparável. As preferências ficam em %LOCALAPPDATA%/AindaVivo/settings.cfg ou AINDA_VIVO_DATA_DIR. Sessões comuns gravam frame_ms no subdiretório sessions. O FPS do smoke usa um relógio virtual de 60 Hz.

## Reproduzir validação

O cache Windows do PC fica em D:/CodexBuilds/ainda-vivo-v0.0.1. O nome da pasta antecede a numeração atual. Use scripts/dev.ps1 com -BuildRoot e -Jobs em outro ambiente. Consulte os requisitos no [README](../README.md).

~~~powershell
cargo fmt --all -- --check
cargo clippy --locked --offline --workspace --all-targets -j 2 -- -D warnings
cargo test --locked --offline --workspace -j 2
cargo build --locked --offline -j 2
New-Item -ItemType Directory -Force captures
& ".\windows\ainda-vivo-v0.0.8.exe" --smoke-test --check-audio --capture-dir ".\captures" --smoke-report ".\smoke-report-v0.0.8.txt"
~~~

Os comandos offline assumem cache disponível. O smoke usa janela e áudio nativos, caminha nos trechos principais e reposiciona a câmera em pontos de teste. O build entregue passou em 29 testes e 12.260 quadros de integração. Foram verificados ambos os adereços ilusórios e as pistas opcionais. Consulte [Validação](VALIDACAO.md) e o [relatório bruto](smoke-report-v0.0.8.txt).

## Próximas etapas

1. Fazer uma sessão às cegas com o diretor do jogo. Medir onde a figura foi percebida, o que os sons sugerem e onde a interação exigiu insistência. Comparar VHS suave e desligado no mesmo trajeto.
2. Gravar CSVs em partidas comuns com configurações equivalentes. Comparar mediana, P95 e P99 de frame_ms nas áreas da Bomba 02, da Decantação e de Controle. Investigar luzes e sombra se a medição apontar gargalo gráfico.
3. Ajustar pesos e pausas com base nos testes e ampliar as variações de cenário ligadas a indícios reconhecíveis.
4. Projetar morte silenciosa separada dos efeitos locais. Android e LAN para 2–4 ficam para mudanças posteriores, depois do próximo corte solo.
