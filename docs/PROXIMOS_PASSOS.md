# Próximos passos

As etapas abaixo registram o planejamento do primeiro protótipo. As prioridades atuais estão na [documentação de continuidade da v0.0.10](../AINDA-VIVO-HANDOFF-v0.0.10.md#problemas-conhecidos-e-continuidade).

1. **Sessão do diretor:** testar 10–15 minutos e anotar conforto, sensibilidade, luz, leitura dos objetos e qualquer colisão estranha. Informar resolução, VSync e FPS observado. Priorizar correções antes de aumentar o mapa.
2. **v0.0.2 — base sólida:** persistir preferências, permitir remapear teclas, registrar tempos de quadro por sessão e ampliar testes de interação/oclusão. Adicionar áudio espacial discreto com volume e opção de desligar.
3. **v0.1 — Presença:** definir comportamento e objetivo narrativo, implementar percepção e navegação verificáveis, morte silenciosa e uma ilusão por jogador. Sem jumpscares baratos nem sacrifício de conforto.
4. **Cenário real:** usar fotos autorizadas pelo diretor para substituir o mapa provisório. Medir escala, portas e geometria de colisão antes de detalhar gráficos. Se houver rampas/escadas, trocar ou ampliar o controlador com testes.
5. **LAN 2–4:** primeiro conexão/entrada/saída de sessão e movimento replicado; depois inventário, portas, criatura e ilusões privadas. Testar desconexão do host, latência e perda de pacotes.
6. **Android:** construir interface e controles de toque e validar num aparelho real; não simplesmente copiar os controles de PC.
7. **Distribuição:** medir o tamanho real antes de decidir sobre pacotes internos. Download depende da conexão e do servidor, não da linguagem Rust. Primeiro distribuir uma versão offline completa e verificável.

Critérios para avançar: build reproduzível, testes passando, sessão sem travar, nenhuma parede atravessável e controles aprovados pelo diretor. O desempenho deve ser medido durante o desenvolvimento, sem prometer requisitos mínimos ou FPS antes de testes reais.
