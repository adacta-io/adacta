import {Component, OnInit} from '@angular/core';
import {Observable} from 'rxjs';
import {InboxService} from '../../shared/services/inbox.service';
import {RepoService} from '../../shared/services/repo.service';

@Component({
  selector: 'app-inbox',
  templateUrl: './inbox.component.html',
  styleUrls: ['./inbox.component.less'],
})
export class InboxComponent implements OnInit {

  constructor(private inbox: InboxService,
              private repo: RepoService) {
  }

  ngOnInit(): void {
  }

  public get docs(): Observable<string[]> {
    return this.inbox.docs;
  }

  public preview(id: string): Observable<string> {
    return this.repo.preview(id);
  }
}
