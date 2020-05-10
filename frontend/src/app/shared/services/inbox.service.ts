import {Injectable} from '@angular/core';
import {HttpClient} from '@angular/common/http';
import {BehaviorSubject, Observable, timer} from 'rxjs';
import {map, shareReplay, switchMap} from 'rxjs/operators';

class InboxResponse {
  count: number;
  docs: string[];
}

@Injectable()
export class InboxService {
  private count$: BehaviorSubject<number | null>;
  private docs$: BehaviorSubject<string[] | null>;

  constructor(private http: HttpClient) {
    this.count$ = new BehaviorSubject(null);
    this.docs$ = new BehaviorSubject(null);

    // TODO: Only refresh if authenticated
    timer(0, 5000)
      .subscribe(() => {
        this.refresh();
      });
  }

  public refresh() {
    this.http.get<InboxResponse>(`/api/inbox`)
      .subscribe(data => {
        this.count$.next(data.count);
        this.docs$.next(data.docs);
      });
  }

  public get count(): Observable<number | null> {
    return this.count$;
  }

  public get docs(): Observable<string[] | null> {
    return this.docs$;
  }
}
