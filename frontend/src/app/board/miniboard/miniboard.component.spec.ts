import { ComponentFixture, TestBed } from '@angular/core/testing';

import { MiniboardComponent } from './miniboard.component';

describe('MiniboardComponent', () => {
  let component: MiniboardComponent;
  let fixture: ComponentFixture<MiniboardComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ MiniboardComponent ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(MiniboardComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
