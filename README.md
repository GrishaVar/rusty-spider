# rusty-spider
A cli [spider](https://en.wikipedia.org/wiki/Spider_(solitaire)) patience game, written in rust.

## Get it running:
* Clone/download the repo
* Launch with `cargo r --release`

## How to play:
* mxz to **M**ove cards from column x to column z
  * Mxyz to move y'th card from column x to column z (when mxz ambiguous)
* s to pull cards from the **S**tack
* c to remove **C**ompleted suits
* z to **U**ndo
* y to **R**edo
* H to show **H**elp menu with other inputs

## Improvements
* Make this a proper cli tool instead of printing the game every time
* Separate inputs with spaces to allow more than 10 numbers
* Abstract to include more patience games
