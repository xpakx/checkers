import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';
import { FormGroup, FormBuilder } from '@angular/forms';
import { GameRequest } from '../dto/game-request';

@Component({
  selector: 'app-new-game-modal',
  templateUrl: './new-game-modal.component.html',
  styleUrls: ['./new-game-modal.component.css']
})
export class NewGameModalComponent implements OnInit {
  requestForm: FormGroup;
  @Output() new: EventEmitter<GameRequest> = new EventEmitter<GameRequest>();
  @Input() ai: boolean = false;

  constructor(private formBuilder: FormBuilder) {
    this.requestForm = this.formBuilder.group({
      username: [''],
      rules: [''],
      ai_type: [''],
    });
   }

  ngOnInit(): void {}

  finish(): void {
    console.log(this.requestForm.value);
    if (this.requestForm.invalid) {
      return;
    }
    let request: GameRequest = {
      "aiType": this.ai ? this.requestForm.value.ai_type : undefined,
      "rules": this.requestForm.value.rules,
      "opponent": this.ai ? undefined : this.requestForm.value.username,
      "type": this.ai ? "AI" : "User"
    }
    this.new.emit(request);
  }
}
