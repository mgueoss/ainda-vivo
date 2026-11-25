# AINDA VIVO — v0.0.11 · Colapso

A vistoria da **Bomba 02** leva o operador à nave antes e depois de rearmar o quadro geral. Quando o fôlego começar a falhar, procure um posto de pausa sinalizado em âmbar. Use **E** no posto para descansar e obter a reserva de foco. Pressione **R** para usá-la, se disponível. [Changelog](CHANGELOG.md) · [Documentação de continuidade com spoilers](AINDA-VIVO-HANDOFF-v0.0.11.md)

O corredor leva à **Estação de Bombeamento 07**, com 15 setores, rotas laterais e cerca de 6.100 m² de planta. A tecla **F4** alterna o VHS entre suave, forte e desligado. O menu da pausa também oferece controle do VHS e do volume. [Detalhes do mapa](docs/ESTACAO-07.md)

Protótipo Windows feito em Rust e Bevy. O usuário dirige o projeto. A prioridade de desenvolvimento é a experiência solo no PC.

## Jogar

Para jogar, baixe o pacote Windows, extraia e abra **AINDA-VIVO.exe**. [ZIP v0.0.11](https://github.com/mgueoss/ainda-vivo/releases/download/v0.0.11/ainda-vivo-v0.0.11-windows-x64.zip) · [GitHub Release v0.0.11](https://github.com/mgueoss/ainda-vivo/releases/tag/v0.0.11) · [Distribuição portátil e dependências](docs/DISTRIBUICAO.md)

No checkout com o binário local, **JOGAR.cmd** abre a v0.0.11. Para compilar pelo fonte e acompanhar mensagens de inicialização, use `scripts/dev.ps1 -Action Run`.

Sem executável pronto, o mesmo atalho compila e executa. A primeira compilação baixa as bibliotecas oficiais do motor; não baixa modelos, texturas, áudio ou pacotes de jogo. Compilações seguintes reaproveitam o cache.

No menu, escolha **Iniciar expedição** com o mouse ou navegue com as setas ou **Tab** e confirme com **Enter**. Pegue o fusível na oficina à esquerda. Use **E** no manômetro da **Bomba 02**, siga as indicações de **ELÉTRICA** e rearme o quadro geral. Volte à mesma bomba para registrar a segunda leitura e liberar a comporta. A saída fica depois da bacia de decantação. O Arquivo e a sala de Controle guardam registros opcionais. [Abertura e controles do menu](docs/MENU-v0.0.10.md)

**Despertar** fica no menu de pausa e pede confirmação. Use **E** no interruptor da luminária para iluminar a mesa do Arquivo.

O jogo contém pânico, alterações sensoriais e colapso sobrenatural. Em **Configurações → Acessibilidade**, ajuste a intensidade audiovisual, reduza movimentos da câmera ou ative o indicador de estabilidade.

| Controle | Ação |
|---|---|
| WASD / mouse | Mover / olhar |
| Shift | Correr |
| Ctrl esquerdo ou C, segurado | Agachar |
| Espaço | Pular |
| E | Interagir com o objeto sob a mira, até 2,4 m |
| F | Ligar/desligar a lanterna |
| R | Usar a reserva de foco |
| Esc | Pausar/continuar e liberar o mouse |
| Enter | Iniciar/continuar |
| F3 | Mostrar/ocultar FPS e diagnóstico |
| F4 | Alternar o modo VHS |

O jogo pausa ao perder o foco. Fechar a janela encerra o programa. O menu permite reiniciar, ajustar sensibilidade e campo de visão e alternar VSync e balanço.

## Conforto e desempenho

- Balanço desligado por padrão. A opção suave desloca a câmera verticalmente em até 5 mm e mantém o horizonte nivelado.
- O VHS simula granulação fina, linhas de fita, separação discreta de cor e leve deslocamento horizontal no rodapé. O efeito age na cena 3D, com o HUD e o menu legíveis. O modo desligado remove o passe gráfico.
- Campo de visão **vertical** ajustável de 55° a 80°; inicial 65°. Sensibilidade ajustável no menu. Preferências são salvas automaticamente.
- Física fixa em 60 Hz, interpolação visual e aceleração suave. VSync ligado inicialmente; FPS desbloqueável no menu.
- Janela inicial de 1280 × 720, redimensionável. Lanterna com sombras, luminárias locais e texturas procedurais.
- Referência de direção: Ryzen 5 5600G, GTX 1660 Super, 16 GB. Não é uma garantia de FPS; medições reais ficam em `docs/VALIDACAO.md`.

## Desenvolvimento

A build experimental `v0.0.12-dev.6` inclui perseguições que se ajustam a desvios, busca com pistas incompletas e ilusões sonoras contextuais. No ZIP experimental, **ABRIR DEV LAB.cmd** habilita a Presença física na campanha. Pressione **Ctrl+I** e abra **IA** para testar desvio, reversão ou circuito. **F5** alterna a representação na campanha quando a entidade se retira. O canal estável segue disponível no GitHub. [Guia do Dev Lab](docs/DEV-LAB.md) · [Handoff de Inteligência Hostil](AINDA-VIVO-HANDOFF-v0.0.12-dev.6.md)

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

Bevy está fixado em **0.18.1**, com as dependências travadas no `Cargo.lock`. O áudio é sintetizado localmente, as texturas são procedurais e o efeito VHS é incorporado no executável. O jogo funciona offline após a compilação.

## Escopo e limitações

O protótipo inclui exploração em primeira pessoa, a vistoria completa, captura silenciosa, Despertar e uma investigação opcional. A câmera fixa da nave produz quadros que podem ser consultados no Controle.

O controlador usa uma caixa cinemática de 54 cm de largura contra caixas estáticas alinhadas aos eixos, com varredura contínua e deslizamento. Não é uma simulação de corpos rígidos: não suporta rampas, escadas automáticas, empurrar objetos, plataformas móveis ou colisores rotacionados. Não inserir essas geometrias sem ampliar e testar a física. Agachar mantém os pés no chão e impede levantar dentro de uma travessa baixa.

O progresso e as memórias ficam na sessão atual. Campanha expandida, salvamento em disco, Android e LAN seguem como próximos passos.

Veja [arquitetura](docs/ARQUITETURA.md), [validação](docs/VALIDACAO.md) e [próximos passos](docs/PROXIMOS_PASSOS.md).

## Referências técnicas

APIs conferidas na [documentação oficial do Bevy 0.18.1](https://docs.rs/bevy/0.18.1/bevy/) e nos exemplos que acompanham essa versão. Recursos ativados: [lista de features](https://docs.rs/crate/bevy/0.18.1/features). A interface lê Consolas ou Segoe UI já instalada no Windows, para mostrar os acentos corretamente; nenhum arquivo dessas fontes é copiado ou distribuído. Se ambas faltarem, usa a fonte limitada do Bevy. Os demais elementos visuais são procedurais.
