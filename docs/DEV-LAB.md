# Dev Lab do AINDA VIVO

A build experimental v0.0.12-dev.2 inclui o Dev Lab. No pacote Windows, extraia os arquivos e execute **ABRIR DEV LAB.cmd**. No menu ou durante a campanha, pressione **Ctrl+I** para entrar ou voltar. O botão de busca recebe a digitação até **Enter**, **Esc** ou um clique em outro controle.

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

## Uso no repositório

```powershell
cargo run --locked --features dev-lab -- --dev-lab
cargo run --locked --features dev-lab -- --dev-lab --lab-campaign-preview
cargo run --locked --features dev-lab -- --dev-lab --lab-assets assets
```

`--lab-campaign-preview` coloca a malha passiva no corredor inicial da Estação, perto do acesso. As aparições atuais e a despedida da v0.0.11 continuam a usar seus recursos originais. O caminho opcional de `--lab-assets` precisa conter `models/presenca.glb`. Nesse modo, **Recarregar** solicita nova leitura do arquivo. A opção `--lab-config CAMINHO` permite usar um arquivo de preferências separado para QA.

Builds sem a feature Cargo `dev-lab` excluem a sala, os controles, o carregador GLB de desenvolvimento e o recurso embutido.
