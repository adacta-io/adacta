import {Component, OnInit, ViewEncapsulation} from '@angular/core';
import {InboxService} from '../../services/inbox.service';
import {ActivatedRoute, Router} from '@angular/router';

@Component({
  selector: 'app-inbox',
  templateUrl: './inbox.component.html',
  styleUrls: ['./inbox.component.less'],
})
export class InboxComponent implements OnInit {

  constructor(private inbox: InboxService) {
  }

  ngOnInit(): void {
  }

  public get docs() {
    return this.inbox.docs;
  }

}
