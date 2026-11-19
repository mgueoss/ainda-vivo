# Distribuição portátil do AINDA VIVO

## Jogar

A v0.0.9 — Falsa Realidade é a versão recomendada para testes. As versões históricas continuam disponíveis no GitHub. [Release v0.0.9](https://github.com/mgueoss/ainda-vivo/releases/tag/v0.0.9) · [Demais releases](https://github.com/mgueoss/ainda-vivo/releases)

Baixe o arquivo `ainda-vivo-vVERSÃO-windows-x64.zip`, extraia o ZIP inteiro e abra **AINDA-VIVO.exe**. O botão **JOGAR** fica disponível após a verificação. A preparação usa o Windows local e o driver de vídeo. Quando falta o Visual C++ x64, o próprio inicializador oferece a instalação oficial da Microsoft. Se o runtime e o driver já estiverem disponíveis, a preparação e o jogo funcionam offline.

~~~text
AINDA-VIVO.exe
game/
  ainda-vivo.exe
  version.txt
dependencies/
LEIA-ME.txt
LICENCAS.txt
~~~

O executável em `game/` tem o mesmo SHA-256 do executável histórico. A versão que aparece na janela do jogo pertence ao próprio jogo. O inicializador só usa o manifesto do pacote para identificar a versão e organizar os registros. Em caso de problema, abra **DIAGNÓSTICO**. Os arquivos `inicio.log` e `jogo.log` ficam em `%LOCALAPPDATA%\AindaVivo\launcher\VERSÃO`. O último contém a saída da sessão mais recente. [Pré-lançamentos do projeto](https://github.com/mgueoss/ainda-vivo/releases)

## Inicializador universal

`crates/ainda_vivo_launcher` é um executável Rust/Win32 independente do Bevy, com interface nativa e runtime próprio ligado estaticamente. Uma tarefa em segundo plano lê os cabeçalhos e nomes da tabela de imports do PE x64. A verificação usa o carregador para DLLs do Windows, incluindo aliases de API, e reconhece bibliotecas x64 ao lado do jogo. Imports tardios ausentes aparecem no diagnóstico. A verificação observa o build do Windows e testa Direct3D 12 no adaptador padrão ou a enumeração de uma GPU Vulkan 1.1. Essa triagem avalia o ambiente. A confirmação de renderização vem da abertura real do jogo.

As versões verificadas importam `VCRUNTIME140.dll` e componentes do Universal CRT do Windows. A lista completa de imports fica no log. O launcher autoriza apenas o reparo dos imports do runtime Visual C++ v14 conhecidos. Depois da confirmação, ele usa o redistribuível assinado que estiver no pacote ou baixa da URL oficial da Microsoft. Ele verifica a assinatura Authenticode e seu publicador antes de iniciar a instalação, verifica novamente o ambiente depois e abre o jogo quando a preparação está concluída. O ZIP padrão usa o runtime instalado no sistema. Há um parâmetro opcional para um mantenedor que tenha licença para redistribuir o instalador oficial e queira incluí-lo para uso offline. [Microsoft sobre redistribuição](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files) · [Downloads oficiais do Visual C++](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist/)

A verificação normal evita leitura integral do executável e não faz chamadas de rede. A assinatura e o download são usados no reparo autorizado. Os SHA-256 integrais são conferidos pelo empacotador e na auditoria das releases. O Windows 10/11 x64 com suporte gráfico ao jogo é o alvo. Sistemas ARM64, Windows anteriores e computadores sem driver de GPU compatível ficam fora do escopo.

## Montar a próxima distribuição

Use **PowerShell 7** para scripts de manutenção. O jogador precisa apenas do executável portátil e dos componentes normais do Windows. No PC original, as variáveis `CARGO_HOME` e `CARGO_TARGET_DIR` podem apontar para o cache do projeto na unidade D:.

~~~powershell
cargo rustc -p ainda-vivo-launcher --profile launcher --locked --offline -- -C target-feature=+crt-static
pwsh -File scripts/distribution/notices.ps1 -OutputFile 'LICENCAS.txt'
pwsh -File scripts/distribution/package.ps1 -Launcher 'CAMINHO_DO_LAUNCHER.exe' -NoticesFile 'LICENCAS.txt' -OutputDirectory 'PASTA_DOS_ZIPS' -Version $Version -GameExe 'BINARIO_VALIDADO.exe'
~~~

`package.ps1` possui um modo de recuperação que lê os oito ZIPs originais com `-OriginalDirectory`. Ele confere os hashes do catálogo e o manifesto histórico, extrai apenas o jogo, verifica a arquitetura e os imports e compara o conteúdo arquivado com as origens. O modo para uma build nova aceita `-AssetsDirectory` se o jogo passar a incluir conteúdo dedicado. Para incluir o redistribuível em uma distribuição licenciada, use `-LicensedRuntimeInstaller`. Gere cada nova distribuição em uma pasta de saída nova.

`notices.ps1` junta os avisos da biblioteca padrão do Rust e da árvore Cargo Windows. Alguns crates declaram licença no manifesto sem trazer uma cópia no arquivo publicado. Para este conjunto são complementares `taffy-0.9.2.txt`, `mit0.txt` e `cc0.txt`. [Taffy MIT](https://github.com/DioxusLabs/taffy/blob/main/LICENSE) · [SPDX MIT-0](https://spdx.org/licenses/MIT-0.html) · [SPDX CC0-1.0](https://spdx.org/licenses/CC0-1.0.html)

`test-packages.ps1` extrai os ZIPs, verifica o SHA-256, aciona o botão nativo e confirma o título, a versão e a inicialização do renderizador de cada jogo no Windows. Ele também verifica o bloqueio de um pacote sem executável. O resultado `validacao.json` é consumido por `publish.ps1`. Para novas versões, acrescente versão, notas, limitações e SHA-256 do binário ao catálogo `versions.json`. Entradas de builds diretas usam `zipSha256: null`. Os scripts de testes e publicação aceitam selecionar uma versão. A tag do código deve conter a mesma versão no manifesto.

Antes da publicação, faça as tags apontarem para as fontes verificadas e envie-as pelo Git junto com o commit da distribuição. `publish.ps1` usa a credencial da conta no Git do PC, verifica os objetos das tags no remoto, exige evidência de inicialização do ZIP exato, cria as releases como rascunhos e verifica os downloads antes e depois de publicar. Releases publicadas recebem verificação de conteúdo, e rascunhos alheios são preservados. As releases incluem ZIP, SHA-256, changelog e limitações.

~~~powershell
pwsh -File scripts/distribution/test-packages.ps1 -PackageDirectory 'PASTA_DOS_ZIPS' -WorkDirectory 'PASTA_DE_TESTE'
pwsh -File scripts/distribution/publish.ps1 -PackageDirectory 'PASTA_DOS_ZIPS' -ValidationReport 'CAMINHO\validacao.json' -OutputReport 'CAMINHO\publicacao.json'
~~~

## Recuperação e evidências

Os ZIPs com fonte são a referência de resgate. `recover-tags.ps1` compara a lista e o conteúdo dos arquivos de código com cada árvore Git. As versões v0.0.1, v0.0.2, v0.0.7 e v0.0.8 correspondem a commits existentes. As versões v0.0.3 a v0.0.6 têm commits de recuperação independentes que identificam a fonte e o SHA-256 do ZIP original, com data real de recuperação e sem uma sequência histórica presumida.

[Catálogo e hashes originais](../scripts/distribution/versions.json) · [Relatório de validação da distribuição](DISTRIBUICAO-VALIDACAO.md) · [Validação do jogo](VALIDACAO.md)
