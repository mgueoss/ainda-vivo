# Validação · AINDA VIVO v0.0.12-dev.5

Ambiente Windows 11 Home x64, AMD Ryzen 5 5600G, GeForce GTX 1660 SUPER, driver NVIDIA 610.88 e Vulkan. Foram usados Bevy 0.18.1 e o perfil `dev` otimizado do projeto. A seed dos testes nativos foi `20261118`.

## Ferramentas e testes unitários

- Formatação conferida com rustfmt e análise estática com Clippy usando `-D warnings`. A compilação experimental e a checagem da configuração sem `dev-lab` passaram.
- Os testes do workspace passaram em Rust. Foram 46 do jogo, 33 do núcleo e 3 do inicializador.
- O banco verifica amplitudes finitas, amostras distintas, caudas e energia. A camada espacial verifica o lado estéreo, a queda de ganho à distância e o filtro de barreira. A caçada invisível tem teste de cadência, aproximação, repetição por seed e categorias com as mesmas famílias.

## Cobertura nativa

| Comando no executável extraído | Resultado | Verificação |
|---|---|---|
| `--smoke-sonoro` | PASS | Aba Som com a mesma fonte atrás da porta fechada e aberta, apoio de mão e pé na corrida, caminhada, voz, caçada invisível, categoria da falsa realidade em laboratório e dispositivo com saída ativa. Confere teto de fontes, limpeza após sair e uma ilusão criada pelo diretor em setor vizinho. |
| `--smoke-hunt` | PASS · 1.390 quadros | Caçada quadrúpede, contato silencioso, fuga por porta, cinco resultados simulados e isolamento do ensaio. |
| `--smoke-observer` e `--smoke-motion` | PASS · 2.880 e 1.520 quadros | Percepção, navegação, importação das animações, pista, transições e contatos. |
| `--smoke-test --check-audio` | PASS · 12.260 quadros | Vistoria da Bomba 02, pausa, colisões, rota de saída, três sons incidentais e emissores da vistoria e da bacia. |
| `--smoke-reality` | PASS · 2.696 quadros | Captura silenciosa, pausa, falsa realidade, Despertar e saída. |
| `--smoke-echoes --check-audio` | PASS · 2.735 quadros | OS 17, retorno da missão e os três emissores espaciais da sequência. |
| `--smoke-collapse` | PASS · 8.548 quadros | Instabilidade, recuperação tardia e despedida. |
| `--smoke-lab` | PASS · 1.492 quadros por execução | Entradas e saídas repetidas, renderização e estabilidade de recursos. |

No teste do som, fechar a porta reduziu a transmissão de 1,00 para 0,51. O ganho de pico entre canais caiu de 0,08976 para 0,04578 e a intensidade do filtro passou de 0,00 para 0,66. O Lab atingiu sete fontes simultâneas e voltou ao menu com suas fontes encerradas. No cenário da Estação, o teste detectou uma fonte da Triagem aos 43,25 s de tempo simulado enquanto o operador estava no Acesso. A caçada física registrou 14 apoios de mãos e 12 de pés na campanha, os mesmos da dev.4 naquele roteiro.

## Desempenho observado

Comparação no mesmo PC em ordem dev.4, dev.5, dev.5 e dev.4, sem compilação concorrente. Foram 1.492 quadros por execução. A mediana abaixo usa as 25 amostras do estúdio coletadas pelo roteiro do Dev Lab.

| Build | Medianas de quadro | Tempo total das duas execuções |
|---|---|---|
| dev.4 | 6,0474 e 6,0022 ms | 10,393 e 10,373 s |
| dev.5 | 6,0478 e 6,0064 ms | 11,148 e 10,405 s |

O tempo de quadro ficou na mesma faixa no estúdio. A amostragem cobre a câmera e a cena desse roteiro. A caçada com sete vozes foi verificada separadamente para funcionamento e limpeza, sem amostragem de percentis de desempenho.

## Integridade do pacote

`ainda-vivo-v0.0.12-dev.5-windows-x64.zip` contém `ainda-vivo.exe`, `ABRIR DEV LAB.cmd`, `LEIA-ME.txt` e `LICENCAS.txt`. Foi extraído em uma pasta isolada, com comparação do SHA-256 do executável produzido e do executável extraído. A cópia de regressão da dev.4 e os arquivos do mapa, modelo e rig permaneceram preservados.

| Arquivo | SHA-256 |
|---|---|
| ZIP dev.5 | `f23d35b3490283af15d971f081585345032cb108a2725c1ed30aaf93a867a2e7` |
| Executável dev.5 | `048f978b5605a93638305d9943ee43217156dca61f3c18d3d550c19819dc109e` |
| ZIP dev.4 preservado | `359d6bb5e29359a04af6d7f5be27c37043afc8201931795bd8485fb71e98e13f` |

## Qualidade sonora e limites

A demonstração estéreo tem seis segundos e usa o banco e o decodificador do jogo. O pico de PCM foi aproximadamente 0,19. Isso verifica margem na sequência de audição. O ensaio nativo confirmou `AudioSink` ativo. A entrada de áudio da sessão estava indisponível para avaliação auditiva. A voz, a leitura espacial em fones e o equilíbrio com a ambiência continuam sujeitos à avaliação humana.

Os testes são roteiros de regressão, gerados por Windows com renderização e dispositivo de áudio. As medidas de tempo descrevem os cenários registrados. O tempo real de quadro depende da carga e da cadência do sistema.
