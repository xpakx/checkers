import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
  name: 'field'
})
export class FieldPipe implements PipeTransform {

  transform(value: String): String {
    if (value == "WhiteKing") {
      return "⛁"
    }
    if (value == "RedKing") {
      return "⛃"
    }
    if (value == "WhitePawn") {
      return "⛀"
    }
    if (value == "RedPawn") {
      return "⛂"
    }
    return "";
  }

}
