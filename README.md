# Writing NES Emulator in Rust

NES エミュレータを題材に Rust を学ぶ  
https://bugzmanov.github.io/nes_ebook/

## Target
* for Linux with SDL2
## References
* ebook sample code
  * https://github.com/bugzmanov/nes_ebook/tree/master/code
* CPU
  * 6502 reference
    * https://www.nesdev.org/obelisk-6502-guide/reference.html
  * unofficial opcodes
    * https://www.nesdev.org/undocumented_opcodes.txt
    * https://www.nesdev.org/wiki/Programming_with_unofficial_opcodes
* PPU
  * https://www.nesdev.org/wiki/PPU
* other
  * https://dev.classmethod.jp/articles/nes-rust/
  * https://scrapbox.io/razokulover-tech-memo/NES

## Memo and comments
* Chapter3.3
  * 相変わらず SBC がバグりやすい
    * NEGAIVE フラグが未だによくわからない
    * 結局 A=A-M-(1-C) -> A=A+(-M-1)+C で ADC と共通処理化
* Chapter3.4
  * game_code の起動アドレスは 0x0600
* Chapter4
  * そのまま書くとテストが全滅するようになる
  * Bus上に起動アドレスを持たせて対応
* Chapter5
  * テスト ROM は Chapter3.4 の game_code がスタートアドレス込みでロードできるようになってるだけ
* Chapter5.1
  * nestest.nesはスタートアドレスが0xC000というか、ROMの先頭から
  * unofficial opcodes
    * SAX、これフラグ変更いらないのか？
    * ALR, ANC, ARR, AXS はこれでいいと思うんだが少し納得がいっていない
  * 一通り CPU 命令実装したが、サンプルのコードは効率が悪い上に見通しが悪いと思う
    * できるだけ test が通るように維持したい
    * ユニットテストだし、その都度だけ通ればいいというものではない
  * nestest.nes、こういうエミュレータ向けテストいいね
    * なんというか、opcode いろは歌的な。
    * Z80 にもほしい
* Chapter 6
  * この時代ならではのビデオチップ。動きが面白い
* Chapter 6.1
  * 一部のレジスタの実装しか書いてないが全部書く
  * テストがめんどくさいが動きが特殊なのでしっかり書く
