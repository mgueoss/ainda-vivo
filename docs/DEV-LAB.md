# Dev Lab do AINDA VIVO

A build experimental v0.0.12-dev.7 inclui o Dev Lab. No pacote Windows, extraia os arquivos, execute **ABRIR DEV LAB.cmd** e clique em **ABRIR DEV LAB** no inicializador. No menu ou durante a campanha, pressione **Ctrl+I** para entrar ou voltar. **JOGAR.cmd** abre a campanha com a Presença em 3D. O botão de busca recebe a digitação até **Enter**, **Esc** ou um clique em outro controle.

## Laboratório Cognitivo

- Abra **IA → Cenário IA** para ver estado do diretor, objetivo, estratégia, confiança, raio da hipótese, plano, utilidades, rotas e memória episódica. Os guias mostram a rota, a região provável e a sonda de visão.
- **Desvio**, **Reversão** e **Circuito** iniciam uma caçada ao bot marcado na pista. A câmera permanece livre para assistir ao ensaio. **Caixa** coloca um obstáculo real quando há espaço. **Porta** alterna a barreira e **Gerar ruído** emite um estímulo físico no alvo selecionado.
- **Pausa IA** suspende o cérebro. **Um passo** avança a simulação em 1/60 s e mantém a pausa. **Limpar memória** apaga pistas e hábitos. **Outra seed** incrementa a seed selecionada e **Repetir seed** reinicia o cenário. Use `--ai-seed 42017` para escolher um valor na abertura.
- **Planejar** ativa a seleção autônoma. Os botões das seis estratégias pedem o plano escolhido quando houver pistas e posição compatíveis. **Reação simulada** alterna entre observação livre, aproximação, indiferença e movimento contrário. **Diretor de ensaio** bloqueia ou permite a entrega de sons solicitados pelo plano. Role o painel para ler a etapa, a hipótese, os retornos e os resultados.
- **Rodar matriz** executa os cenários determinísticos e exibe o resultado. **Exportar diagnóstico** salva `inteligencia-presenca.txt` na pasta das capturas do Lab, incluindo as rejeições de oportunidades da campanha. Os modos `--check-cognition` e `--check-strategy` escrevem ensaios em `--smoke-report CAMINHO` sem carregar o renderizador.

Na campanha com a Presença física, use **E** no relé da Elétrica quando aparecer **conferir via e testar o relé**. O estalo sai do equipamento cerca de 1,45 s depois. O operador ganha tempo para mudar de direção ou ficar em silêncio. A criatura pode investigar o estímulo se conseguir ouvi-lo. A conferência narrativa do relé continua disponível.

## Som

- **Biblioteca da Presença** toca cada família. **Variação** percorre três amostras e **Repetir** toca de novo. **Categoria** permite comparar manifestação, ilusão e falsa realidade com a mesma amostra.
- **Posição** move a fonte de teste para frente, esquerda, direita ou atrás da porta. Mude a distância pelos botões **−** e **+** ou desloque a própria câmera. **Porta** monta o corredor de teste e alterna a barreira. Compare a mesma passada com a porta aberta e fechada.
- **Caminhada** e **Corrida física** usam as animações e seus marcadores. **Caçada invisível** usa os mesmos sons em movimento sem precisar de criatura física. **Interromper sequência** suspende os novos pulsos da simulação, deixando as caudas curtas terminarem.
- **Reverberação** seleciona a amostra com reflexões curtas. **Abafamento** ativa a consulta aos obstáculos. **Guias** mostra um marcador na última origem com anéis fixos de referência. Os percentuais no painel mostram ganho do canal mais forte, transmissão e filtro da fonte ativa mais audível.
- As fontes respeitam as opções globais de ambiência e intensidade audiovisual. Use `--smoke-sonoro --seed 20261118 --smoke-report resultado.txt` para a rotina automatizada com dispositivo de áudio. [Arquitetura do áudio](AUDIO-PRESENCA-v0.0.12-dev.5.md)

| Atalho | Ação |
|---|---|
| Ctrl+I | Entrar ou voltar à sessão anterior |
| Ctrl+H | Ocultar ou mostrar a interface |
| Ctrl+G | Alternar grade e régua |
| Ctrl+L | Avançar para o próximo preset de iluminação |
| Ctrl+B | Exibir ou ocultar o esqueleto da Presença selecionada |
| Ctrl+W | Exibir ou ocultar o wireframe do modelo selecionado |
| Ctrl+P | Reproduzir ou pausar o clipe selecionado |
| Ctrl+R | Restaurar a cena e as preferências padrão em memória |
| Ctrl+F | Trocar entre primeira pessoa e câmera livre |
| Ctrl+Shift+S | Capturar o quadro em PNG |
| Tab ou clique na cena | Capturar o mouse |
| Tab ou Esc | Liberar o mouse |
| WASD, mouse e Shift | Deslocamento, olhar e velocidade |
| Espaço e Ctrl | Subir e descer na câmera livre. Na primeira pessoa, pular e agachar |
| Roda do mouse | Aproximar a câmera em órbita ou rolar um painel sob o cursor |
| F | Usar a lanterna do laboratório |

Na biblioteca, a busca combina o nome e a categoria. Clique em um item para adicionar uma instância. **Comparar** monta uma cena com a nova Presença, o recorte da máscara antiga e um manequim de 1,78 m. As ações de selecionar, duplicar e remover operam sobre a cena de laboratório. O catálogo inicial reaproveita as malhas da Bomba 02, da comporta e de um pilar da nave.

**Objeto** permite deslocar, girar, redimensionar, ocultar e trocar a leitura do material. Os dados técnicos usam a malha importada. O guia **Colisor** mostra as caixas de referência da postura em pé e quadrúpede. **Luz** tem estúdio, Estação 07, corredor escuro, emergência e só lanterna, além de controles de direção, cor, preenchimento, exposição, névoa, sombras e VHS. Para aproximar rosto ou mãos, escolha a pose imóvel e use **Objeto → Rosto** ou **Mãos**.

## Animação

- Selecione a Presença e abra **Animação**. A lista reúne os 39 clipes do GLB. Use **Reproduzir**, **Reiniciar** ou **Loop**. A barra permite ir a um ponto do clipe. Os traços indicam eventos, e **−1 quadro** e **+1 quadro** pausam a reprodução e movem uma amostra de origem a 30 Hz.
- **Apoios** projeta mãos e pés no piso. Verde indica a janela de apoio autorada, e laranja indica um membro em transição. A linha vertical liga a projeção ao osso. **Ossos** e **Wireframe** ajudam a inspecionar deformações.
- **Velocidade do clipe** vai de 0,1× a 3×. **Deslocar** aplica o avanço previsto pelo clipe no espaço, e o controle **Deslocamento** ajusta esse avanço entre 0× e 2×. Use 1× para comparar a passada com o deslocamento para o qual ela foi animada.
- **Comparar A/B** coloca duas instâncias no estúdio. Selecione cada uma na biblioteca, escolha seu clipe e avance até a pose desejada. A instância fora da seleção conserva sua pose.
- **Olhar para o operador** escolhe o clipe de acompanhamento e conduz a amostra de acordo com a posição da câmera diante da criatura. Para testar, mova-se lateralmente entre vistas frontais.
- **Sequência** encadeia observação, espiada, ruptura, vigília e retorno. **Pista** percorre a faixa marcada da sala com aceleração, curva e desaceleração. **Aproximação** coloca a câmera no ponto de observação em primeira pessoa. A Presença corre até uma marca à frente e permanece imóvel. O operador pode se mover ou mudar a iluminação durante o teste. A demonstração ignora dano, captura e sanidade, mesmo quando iniciada a partir da campanha.
- **Encerrar demonstração** devolve o modelo ao ponto de inspeção da cena. Para voltar à câmera inicial, use **Ctrl+R**. Encerrar o Lab preserva a câmera e o progresso da campanha.

**Salvar cena** grava as preferências em `%LOCALAPPDATA%\AindaVivo\dev-lab\settings.json`. As capturas ficam em `%LOCALAPPDATA%\AindaVivo\dev-lab\captures`. Restaurar os padrões vale para a sessão corrente. Salvar depois torna essa escolha persistente. As alterações de instâncias e materiais no Lab são locais à cena.

## IA e navegação

Abra **IA → Cenário IA** para usar o corredor leste. **Automática** alterna a tomada de decisões. Os botões de estado manual permitem dirigir o comportamento. **Pausa IA** congela as decisões e a animação. **Reiniciar** repõe a criatura e limpa a memória. A câmera e os comandos de movimento do laboratório continuam disponíveis.

- **Porta** alterna a obstrução física do corredor. Se houver um corpo na abertura, a porta espera por espaço desocupado.
- **Próximo alvo** muda a posição marcada no piso. **Gerar ruído** produz um estímulo nessa posição. Use **Investigação** na lista manual para ordenar o deslocamento explícito ao alvo.
- **Guias** mostram o cone e os raios, o corpo, a rota e os destinos. Verde marca a rota, os pontos e um raio confirmado. Âmbar marca o cone, a caixa corporal e os raios não confirmados.
- Ajuste alcance, abertura da visão, sensibilidade, velocidade e tempos em **Parâmetros**. As leituras de memória têm coordenadas X/Z em metros, com o tempo restante em segundos.

Para testar na Estação, use `ainda-vivo.exe --presence-physical` ou abra a build com **ABRIR DEV LAB.cmd** e pressione **F5** fora do Lab. A alternância espera o encontro ativo terminar. O terminal pode fixar a seed com `--seed 20261118`.

## Caçada

- **Base**, **Moderada** e **Intensa** trocam em conjunto velocidade máxima, aceleração e orçamentos de ataque e busca no Lab. É possível modificar cada parâmetro após selecionar o perfil. Uma alteração vale para o próximo ensaio.
- **Caçada livre** inicia no lado oeste. Use **Tab** para capturar o mouse, **WASD** para correr para uma esquina e **Shift** para sprintar. **Ameaça** ensaia a ruptura com retorno em pé. **Interromper** pede freada e retirada.
- **Teste de porta** usa o corredor leste. Alterne **Porta** durante a corrida para cortar a linha de visão. Os guias da aba **IA** também aparecem neste cenário.
- Ajuste velocidade, aceleração, distância inicial, preparação, limite de tempo e alcance de contato. Os valores são aplicados à próxima caçada. O teste de porta restringe a distância inicial ao espaço do corredor.
- Escolha **Contextual** ou um dos cinco resultados. **Simular contato selecionado** ensaia o desfecho diretamente. O painel mostra a distância, a velocidade, a rota e a razão de desistência. **Consequências psicológicas** alterna o acúmulo de instabilidade e a perda breve de firmeza dos controles no Lab. A sessão de ensaio tem seu próprio ponto de Despertar e progresso.
- Na campanha experimental, **F5** alterna a representação como na dev.3. A perseguição pode ocorrer depois da leitura inicial da Bomba 02, após uma oportunidade do diretor. Os postos de pausa e a luminária de refúgio interrompem a perseguição.

## Uso no repositório

```powershell
cargo run --locked --features dev-lab -- --dev-lab
cargo run --locked --features dev-lab -- --dev-lab --lab-campaign-preview
cargo run --locked --features dev-lab -- --dev-lab --lab-assets assets
```

`--lab-campaign-preview` coloca a malha passiva no corredor inicial da Estação, perto do acesso. As aparições atuais e a despedida da v0.0.11 continuam a usar seus recursos originais. O caminho opcional de `--lab-assets` precisa conter `models/presenca.glb`. Nesse modo, **Recarregar** solicita nova leitura do arquivo. A opção `--lab-config CAMINHO` permite usar um arquivo de preferências separado para QA.

Builds sem a feature Cargo `dev-lab` excluem a sala, os controles, o carregador GLB de desenvolvimento e o recurso embutido.
