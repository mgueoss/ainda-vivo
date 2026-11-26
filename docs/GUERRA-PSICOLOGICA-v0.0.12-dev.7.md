# Guerra Psicológica

A dev.7 parte do commit auditado `00f69f0b6f5ad43027b8e0ce6e252891a4b27c45`. O mapa, o esqueleto, o GLB, os clipes, os sons e os desfechos usam os recursos da dev.6. A Presença física agora é inicializada por `presence::Runtime` também sem a feature `dev-lab`. A silhueta é selecionável com `--presence-legacy` e serve de contingência se o catálogo físico falhar.

## Decisão e movimento

`src/presence/observer/strategy.rs` pontua seis estratégias com base na pista disponível, distância, posição própria, cobertura, portais, memória e resultados recentes. O candidato precisa superar o limiar de utilidade e construir um plano válido. O cérebro solicita ações táticas de locomoção, espera, apresentação e avaliação. A caminhada de `Brain` e o motor de `Hunt` continuam executando os deslocamentos e as transições de animação de forma mutuamente exclusiva. Paredes, porta e rebaixo são verificados com os volumes existentes.

| Estratégia | Ações e resultado observável |
|---|---|
| Desvio de atenção | Solicitar som num setor vizinho, avançar a uma posição com cobertura, observar e julgar a travessia ou direção que conseguiu ver. Pode expirar, esbarrar no diretor ou falhar se o operador segue outro caminho, percebe a criatura ou ignora o som de forma observável. |
| Interceptação | Estimar o portal mais provável pelo rumo e pelas travessias observadas. Somente considera uma cabeça de rota próxima o bastante para ter chance de chegar primeiro. A ida é a pé e o plano pode escolher a saída errada. |
| Falsa retirada | Afastar-se, parar brevemente e sondar a hipótese anterior. Somente um novo indício pode confirmar que vale a pena voltar a observar. Se a rota bloquear ou a busca vencer, o cérebro volta às decisões locais. |
| Emboscada paciente | Selecionar um ponto próximo com cobertura e linha para o portal provável, deslocar-se até ele e esperar cerca de 6,5 s. O deslocamento enfrenta colisores. Uma espera sem evidência perde prioridade. |
| Perseguição enganosa | Pedir uma aproximação sonora em outro setor, ameaçar e observar. Uma caçada pode começar durante a espera se houver nova visão válida, espaço e janela do diretor. |
| Investigação | Voltar à pista recente e, se a passagem provável for próxima, sondá-la brevemente. Para pistas distantes, o primeiro trecho é limitado a 7,6 m e aponta a uma passagem conectada ou a uma rota direta no mesmo setor. Recuperar visão confirmada encerra a busca. |

Cada plano tem três a cinco ações, prazo derivado do custo e limitado entre 6 e 20 s, hipótese com confiança e raio, e condições de interrupção. A memória da tentativa preserva pistas de visão por 9 s, sons por 8 s, 12 episódios e 12 hábitos dirigidos com horizonte de 88 s. Cada estratégia recebe um viés limitado entre −0,55 e +0,23 com base em até 12 resultados e decaimento ao longo de 110 s. Bloqueio e reação inesperada pesam mais que ausência de indícios. Pausa de apresentação, proteção e início de caçada não penalizam a hipótese. Ao trocar de tentativa ou revisão perceptiva o cérebro da campanha é recriado.

## Coordenação perceptiva

- O planejador recebe pistas que sobreviveram aos sensores de visão e audição. Seu contrato pode indicar tipo de ilusão, setor aproximado, setor do ator e intensidade. Uma solicitação da campanha carrega o identificador da tentativa e da revisão perceptiva, com janela de 0,08 a 2,25 s.
- O diretor arbitra a janela e o áudio reutiliza respiração, passos e corrida. A primeira chance para pedidos estratégicos ocorre após 18 s, com intervalo de 29 s após uma reserva. Uma faixa de apresentação tem um único ocupante. Proteção, tom da cena, leitura, caçada e falta de fonte no setor indicado podem rejeitar ou cancelar um pedido.
- O retorno ao cérebro indica reserva, emissão, rejeição ou interrupção. O código de solicitação não carrega posição real nem resultado da reação. Para atribuir a reação é preciso nova visão com confiança suficiente. A origem do som pode ser a mesma na realidade normal e na falsa. Só ruídos físicos, como passos e o relé, entram no sensor acústico do ator.
- O campo `noticed` significa exposição geométrica ao olhar do operador. Ele reduz a chance de uma distração bem-sucedida e não mede o medo de quem joga. Os eventos perceptivos só afetam apresentação e não alteram os objetivos, a malha de colisão ou as regras de Despertar.

## Contrajogo e caçada

O relé da Elétrica dispara uma vez por acionamento, com atraso de 1,45 s e intervalo de 24 s entre armamentos. Andar agachado produz passos mais fracos e espaçados. Cortar a linha de visão com paredes ou portas, alternar passagens e usar os postos de pausa ou a luminária do Arquivo reduzem as ocasiões de contato. Se a postura baixa não aceita a passagem e a rota em pé cruza uma boca classificada como estreita, a corrida termina com `End::Narrow`. O ator freia, levanta e investiga a pé quando ainda tem uma pista válida. Em caso de bloqueio completo pode desistir.

A campanha usa o perfil base do Lab. Velocidade do operador 4,50 m/s, da Presença 4,95 m/s, aceleração 4,20 m/s², ataque de 7,5 s e corrida com busca de até 12 s. O diretor começa a considerar oportunidades aos 36 s na fase Elétrica ou posterior, a distâncias de 5,5 a 17,2 m. O intervalo de amostragem é 0,35 s. É preciso ter visão recente, postura viável e duas direções locais de fuga. A preparação dura de 1,65 a 3 s. Uma janela elegível tem 38% de chance de aprovação, ou 49% se a velocidade observada supera 2,7 m/s. Sorteio recusado impõe intervalo de 15 s. Uma oportunidade aprovada fecha a janela por 115 a 163 s e tem 36% de chance de ser blefe. Outras restrições também podem impedir o início.

Os perfis de 5,50 m/s e 6,00 m/s existem apenas para comparação no Lab. Permanecem os cinco resultados contextuais de contato e os limites psicológicos da base. A captura continua condicionada à elegibilidade narrativa. Despertar pode recuperar o posto registrado ou encerrar a tentativa conforme a realidade autoritativa.

## Leitura dos resultados

As demonstrações reproduzíveis e as campanhas naturais têm protocolos distintos. Uma saída completa e uma emboscada reproduzível sustentam conclusões diferentes. Consulte o [relatório de testes](VALIDACAO-v0.0.12-dev.7.md) e o [relatório consolidado](../AINDA-VIVO-RELATORIO-GUERRA-PSICOLOGICA.txt) para ver os limites observados.
