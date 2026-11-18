# Validação da distribuição Windows x64

**18 de novembro de 2026 · Windows x64 build 26200 · NVIDIA GeForce GTX 1660 SUPER**

Foram recuperados oito ZIPs originais. Cada SHA-256 do executável coincidiu entre o ZIP original, a cópia histórica local e o pacote portátil extraído. Os manifestos e títulos dos jogos coincidiram com a versão selecionada. O Bevy inicializou o renderizador Vulkan em todas as oito janelas. O teste iniciou as versões pelo evento nativo do botão **JOGAR** e fechou as próprias janelas. A captura documenta o estado pronto do inicializador. [Interface testada](imagens/launcher.png)

| Versão | Fonte da tag | Abertura | Verificação local | SHA-256 do pacote portátil |
|---|---|---|---:|---|
| v0.0.1 | `b3d9af1` | PASS | 485 ms | `2f29d4cb059d5513696696e9166844b6d5e47e58dca93e7ed4dd3ffd1db64d91` |
| v0.0.2 | `58e641c` | PASS | 348 ms | `ca798e8c7c29a57c0acd240d83dc42fc2a34146c44378eddc0681fa559ba3886` |
| v0.0.3 | `ba394a3` | PASS | 352 ms | `04b3aab33d29a273431abbae1dc6da5edae5a2b3c3df8a3d43743255e29355a3` |
| v0.0.4 | `cabd626` | PASS | 345 ms | `f34f374afde1e0410a204317041ac4d258c43b50befb631fdc6320afe352d14c` |
| v0.0.5 | `53b5887` | PASS | 348 ms | `c33dd389f372931a76fab0d556bf59a560b5b5edef99efbb525371328202da8c` |
| v0.0.6 | `c0a2712` | PASS | 389 ms | `ed8a657bdaf89b286cef7d1455238f2b93ed74f9c1151429a1f7fea1b8f998ea` |
| v0.0.7 | `6065b69` | PASS | 386 ms | `92f65737ad35f41c8de926ab0a7b6b2073093eae3faa338f59679b11fc469729` |
| v0.0.8 | `24f782f` | PASS | 369 ms | `d4e2b2a85b5c690ae55bda4e65778f712d9d5ec382533f73306dcd2d08ded257` |

A duração é da verificação local em processo separado nesta máquina. Ela inclui criar o processo e consultar dependências e GPU. O jogo começa depois do botão JOGAR. Essa medição não é um benchmark de jogo.

- Todos os pacotes contêm o mesmo launcher de 467.968 bytes, os dois arquivos do jogo, os avisos de terceiros, instruções e diretório de dependências vazio. O launcher foi auditado sem imports de Visual C++ ou Universal CRT externos.
- Os 20 imports dos jogos incluem `VCRUNTIME140.dll` e aliases de API do Windows. Os oito executáveis passaram na triagem pelas DLLs reais. Windows, Vulkan e Direct3D 12 foram reconhecidos. Nenhum download foi iniciado durante as verificações prontas.
- Rustfmt, Clippy com `-D warnings` e os três testes do inicializador passaram. O caso de runtime ausente foi simulado em teste unitário. Um pacote sem jogo retornou o estado bloqueado e código 1 em execução real.
- O runtime já estava instalado neste computador. O download, o consentimento com elevação e a instalação real do redistribuível ficam para um teste em instalação limpa. Os pacotes padrão permitem jogar offline quando o runtime e o driver já estão disponíveis.
- A validação atual cobre empacotamento, integridade, abertura e inicialização do renderizador. Ela não repete todos os testes de jogabilidade de cada versão histórica. [Validação histórica](VALIDACAO.md)

As tags v0.0.3, v0.0.4, v0.0.5 e v0.0.6 apontam para snapshots recuperados dos respectivos ZIPs, com autoria e data da recuperação. As demais apontam para commits encontrados no histórico. [Catálogo dos ZIPs e executáveis originais](../scripts/distribution/versions.json)
