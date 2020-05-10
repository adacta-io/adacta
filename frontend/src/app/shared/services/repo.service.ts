import {Injectable} from '@angular/core';
import {HttpClient} from '@angular/common/http';
import {Observable} from 'rxjs';
import {flatMap} from 'rxjs/operators';


@Injectable()
export class RepoService {
  constructor(private http: HttpClient) {
  }

  public document(id: string): Observable<Blob> {
    return this.http.get(`/api/repo/${id}/document`, {responseType: 'blob'});
  }

  public preview(id: string): Observable<string> {
    return this.http.get(`/api/repo/${id}/preview`, {responseType: 'blob'}).pipe(
      flatMap(this.readFile)
    );
  }

  private readFile(blob: Blob): Observable<string> {
    return new Observable(obs => {
      const reader = new FileReader();
      reader.onerror = err => obs.error(err);
      reader.onabort = err => obs.error(err);
      reader.onload = () => obs.next(reader.result as string);
      reader.onloadend = () => obs.complete();

      return reader.readAsDataURL(blob);
    });
  }
}
