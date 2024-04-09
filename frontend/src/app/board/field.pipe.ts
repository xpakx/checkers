import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
  name: 'field'
})
export class FieldPipe implements PipeTransform {

  transform(value: String): String {
    if (value == "WhiteKing") {
      return "⛁"
    }
    if (value == "BlackKing") {
      return "⛃"
    }
    if (value == "White") {
      return "⛀"
    }
    if (value == "Black") {
      return "⛂"
    }
    return "";
  }

}
