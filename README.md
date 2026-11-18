# AINDA VIVO — v0.0.3

O corredor ganhou áudio direcional, iluminação por áreas e objetos de uma oficina técnica. Use fones para perceber de onde vêm os sons. No menu e na pausa, os botões **Som −**, **Som +** e **Silenciar** ajustam a ambiência e salvam sua escolha. [Pesquisa e decisões da ambientação](docs/AMBIENTACAO.md)

Protótipo real para Windows, feito em Rust + Bevy. Direção: usuário. Programação: IA, com decisões registradas aqui. PC primeiro; Android e LAN depois.

## Jogar

Abra **JOGAR.cmd** para iniciar `windows/ainda-vivo-v0.0.3.exe` offline. Para acompanhar e salvar mensagens de inicialização, use `scripts/dev.ps1 -Action Run`.

Sem executável pronto, o mesmo atalho compila e executa. A primeira compilação baixa as bibliotecas oficiais do motor; não baixa modelos, texturas, áudio ou pacotes de jogo. Compilações seguintes reaproveitam o cache.

No menu, clique em **INICIAR EXPEDIÇÃO** ou aperte **Enter**. Encontre o fusível na mesa da sala à esquerda, mire nele e use E. Leve-o ao quadro elétrico no fim do corredor, use E e atravesse a saída. A sala à direita tem um registro opcional. O cenário é provisório, sem relação com um local real.

| Controle | Ação |
|---|---|
| WASD / mouse | Mover / olhar |
| Shift | Correr |
| Ctrl esquerdo ou C, segurado | Agachar |
| Espaço | Pular |
| E | Interagir com o objeto sob a mira, até 2,4 m |
| F | Ligar/desligar a lanterna |
| Esc | Pausar/continuar e liberar o mouse |
| Enter | Iniciar/continuar |
| F3 | Mostrar/ocultar FPS e diagnóstico |

O jogo pausa ao perder o foco. Fechar a janela encerra o programa. O menu permite reiniciar, ajustar sensibilidade e campo de visão e alternar VSync e balanço.

## Conforto e desempenho

- Balanço desligado por padrão. A opção suave desloca a câmera verticalmente em no máximo 6 mm; não inclina o horizonte.
- Sem motion blur, distorção de lente, aberração cromática, granulação, flashes ou mudança de FOV ao correr. O estilo de gravação usa enquadramento, luz e interface.
- Campo de visão **vertical** ajustável de 55° a 80°; inicial 65°. Sensibilidade ajustável no menu. Preferências são salvas automaticamente.
- Física fixa em 60 Hz, interpolação visual e aceleração suave. VSync ligado inicialmente; FPS desbloqueável no menu.
- Janela inicial de 1280 × 720, redimensionável. Lanterna com sombras, luminárias locais e texturas procedurais.
- Referência de direção: Ryzen 5 5600G, GTX 1660 Super, 16 GB. Não é uma garantia de FPS; medições reais ficam em `docs/VALIDACAO.md`.

## Desenvolvimento

Requisitos: Windows x64, Rust estável com Cargo, Visual Studio C++ Build Tools e Windows SDK. O ambiente original tem Rust/Cargo 1.97.1 e Visual Studio 2022 Community. Não é necessário instalar Blender para esta versão.

Na pasta do projeto, usando PowerShell:

```powershell
.\scripts\dev.ps1 -Action Run
.\scripts\dev.ps1 -Action Build
.\scripts\dev.ps1 -Action Test
.\scripts\dev.ps1 -Action Check
.\scripts\dev.ps1 -Action Smoke
```

Para reduzir a carga durante a revisão ou compilação, acrescente `-Jobs 1` ao comando do script.

O script usa `D:\CodexBuilds\ainda-vivo-v0.0.1` quando D: existe. No PC original, C: tinha apenas cerca de 600 MB livres; cache, compilação, temporários e logs ficam no D:. Sem D:, usa `target/local-build` no próprio projeto. Reserve alguns GB para compilar, além do espaço do compilador. Para escolher outra pasta **nova ou pertencente ao projeto**:

```powershell
.\scripts\dev.ps1 -Action Run -BuildRoot 'E:\MeusBuilds\AindaVivo'
```

Os comandos Cargo convencionais também funcionam, desde que haja espaço no disco do cache e de `target`:

```powershell
cargo run --locked
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Bevy está fixado em **0.18.1**, com as dependências travadas no `Cargo.lock`. O áudio é sintetizado localmente e reproduzido pelo motor no dispositivo padrão do Windows. O jogo funciona offline após a compilação.

## Escopo e limitações

O protótipo inclui exploração, corrida, agachamento, pulo, colisões, lanterna, interação, um objetivo curto, menu, pausa e diagnóstico. Durante a expedição, a iluminação e a ambiência reagem ao retorno da oficina e à energia restaurada. A Presença ainda aparece como uma silhueta distante e estática.

O controlador usa uma caixa cinemática de 54 cm de largura contra caixas estáticas alinhadas aos eixos, com varredura contínua e deslizamento. Não é uma simulação de corpos rígidos: não suporta rampas, escadas automáticas, empurrar objetos, plataformas móveis ou colisores rotacionados. Não inserir essas geometrias sem ampliar e testar a física. Agachar mantém os pés no chão e impede levantar dentro de uma travessa baixa.

Sem salvamento de progresso, multiplayer, Android, calibração automática, instalador de conteúdo ou download interno. Ilusões individuais, morte silenciosa e campanha são diretrizes futuras, não funcionalidades já implementadas.

Veja [arquitetura](docs/ARQUITETURA.md), [validação](docs/VALIDACAO.md) e [próximos passos](docs/PROXIMOS_PASSOS.md).

## Referências técnicas

APIs conferidas na [documentação oficial do Bevy 0.18.1](https://docs.rs/bevy/0.18.1/bevy/) e nos exemplos que acompanham essa versão. Recursos ativados: [lista de features](https://docs.rs/crate/bevy/0.18.1/features). A interface lê Consolas ou Segoe UI já instalada no Windows, para mostrar os acentos corretamente; nenhum arquivo dessas fontes é copiado ou distribuído. Se ambas faltarem, usa a fonte limitada do Bevy. Os demais elementos visuais são procedurais.
