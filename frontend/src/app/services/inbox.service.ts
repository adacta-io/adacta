import {Injectable} from '@angular/core';
import {HttpClient} from '@angular/common/http';
import {Observable, timer} from 'rxjs';
import {map, share, shareReplay, switchMap} from 'rxjs/operators';

class InboxResponse {
  count: number;
  docs: string[];
}

@Injectable({
  providedIn: 'root'
})
export class InboxService {
  public count: number;
  public docs: string[];

  constructor(private http: HttpClient) {
    timer(0, 5000)
      .pipe(
        switchMap((value, index) => this.http.get<InboxResponse>(`/api/inbox`)),
      ).subscribe(inbox => {
        this.count = inbox.count;
        this.docs = inbox.docs;
      });
  }
}
