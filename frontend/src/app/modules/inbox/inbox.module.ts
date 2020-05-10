import {NgModule} from '@angular/core';
import {CommonModule} from '@angular/common';

import {ClarityModule} from '@clr/angular';
import {InboxRoutingModule} from './inbox-routing.module';
import {ReviewComponent} from './review.component';
import {InboxComponent} from './inbox.component';
import {EmptyComponent} from './empty.component';
import {PdfJsViewerModule} from 'ng2-pdfjs-viewer';
import {SharedModule} from '../../shared/shared.module';


@NgModule({
  declarations: [
    InboxComponent,
    ReviewComponent,
    EmptyComponent
  ],
  imports: [
    CommonModule,
    InboxRoutingModule,
    ClarityModule,
    SharedModule,
  ],
})
export class InboxModule {
}
