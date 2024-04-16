import { Injectable } from '@angular/core';
import { Subject, Observable } from 'rxjs';
import { Toast } from './dto/toast';

@Injectable({
  providedIn: 'root'
})
export class ToastService {
  private toastSubject: Subject<Toast> = new Subject<Toast>();
  toast$: Observable<Toast> = this.toastSubject.asObservable();

  constructor() { }

  createToast(toast: Toast) {
    this.toastSubject.next(toast);
  }
}
