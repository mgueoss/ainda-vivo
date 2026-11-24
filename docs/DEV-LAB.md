# Dev Lab do AINDA VIVO

A build experimental v0.0.12-dev.1 inclui o Dev Lab. No pacote Windows, extraia os arquivos e execute **ABRIR DEV LAB.cmd**. No menu ou durante a campanha, pressione **Ctrl+I** para entrar ou voltar. O botão de busca recebe a digitação até **Enter**, **Esc** ou um clique em outro controle.

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

**Objeto** permite deslocar, girar, redimensionar, ocultar e trocar a leitura do material. Os dados técnicos usam a malha importada. Colisores existentes aparecem como guias. **Luz** tem estúdio, Estação 07, corredor escuro, emergência e só lanterna, além de controles de direção, cor, preenchimento, exposição, névoa, sombras e VHS. **Poses** lista os clipes encontrados no GLB. O cursor de reprodução pode avançar ou retroceder em passos de 0,1 s e ir a frações do clipe. Para aproximar rosto ou mãos, escolha a pose imóvel e use **Objeto → Rosto** ou **Mãos**.

**Salvar cena** grava as preferências em `%LOCALAPPDATA%\AindaVivo\dev-lab\settings.json`. As capturas ficam em `%LOCALAPPDATA%\AindaVivo\dev-lab\captures`. Restaurar os padrões vale para a sessão corrente. Salvar depois torna essa escolha persistente. As alterações de instâncias e materiais no Lab são locais à cena.

## Uso no repositório

```powershell
cargo run --locked --features dev-lab -- --dev-lab
cargo run --locked --features dev-lab -- --dev-lab --lab-campaign-preview
cargo run --locked --features dev-lab -- --dev-lab --lab-assets assets
```

`--lab-campaign-preview` coloca a malha passiva no corredor inicial da Estação, perto do acesso. As aparições atuais e a despedida da v0.0.11 continuam a usar seus recursos originais. O caminho opcional de `--lab-assets` precisa conter `models/presenca.glb`. Nesse modo, **Recarregar** solicita nova leitura do arquivo. A opção `--lab-config CAMINHO` permite usar um arquivo de preferências separado para QA.

Builds sem a feature Cargo `dev-lab` excluem a sala, os controles, o carregador GLB de desenvolvimento e o recurso embutido.
