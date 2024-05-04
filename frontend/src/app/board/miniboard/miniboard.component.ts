import { Component, Input, OnInit } from '@angular/core';
import { Game } from 'src/app/main/dto/game';

@Component({
  selector: 'app-miniboard',
  templateUrl: './miniboard.component.html',
  styleUrls: ['./miniboard.component.css']
})
export class MiniboardComponent implements OnInit {
  @Input() game?: Game; 

  constructor() { }

  ngOnInit(): void {
  }

}
