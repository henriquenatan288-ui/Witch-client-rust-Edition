# Witch Client — Rust Edition

Client de mods para o **Horizon / Starling** (Minecraft reescrito em Rust), feito como *native trait mod*.

## Estado (honesto)
- `witchcore/`: o ClickGUI (abas, lista de modulos, ligar/desligar, scroll). **Testado**: `cd witchcore && cargo test` (7 testes passam).
- `engine/`: liga o ClickGUI aos hooks do Horizon (abre com **Right Shift**). **Nao compilado ainda** — o launcher gera o trait `Hooks` no build; os pontos incertos estao marcados com `CONFERIR`.
- A moldura do menu e uma imagem sua (`assets/frame.png`, 640x356, fundo transparente), enviada ao atlas do HUD com `mod_image` e desenhada com `sprite`. Se ela falhar, o menu cai para retangulos.
- Modulos com logica real (testada em `witchcore`): Player ESP, Tracers, Block ESP (grade de blocos), Fly, Trigger Bot. A parte que os liga ao jogo (`engine/`) nao foi compilada.
- Os demais da lista so guardam ligado/desligado.

## Instalar
`horizon-launcher native-build <esta pasta> --instance <instancia> --trust-native-code`
Depois abra o mundo e aperte Right Shift (no celular, ainda nao ha botao para abrir).

## Proximos passos
1. Compilar no launcher e corrigir o que o `CONFERIR` apontar.
2. Botao de toque para abrir o menu no Android.
3. Primeiros modulos reais: FPS, Coordinates, Radar/ESP (API `native_client_entities`).
