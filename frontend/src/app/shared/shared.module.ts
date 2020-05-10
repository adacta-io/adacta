import {NgModule} from '@angular/core';
import {CommonModule} from '@angular/common';
import {HTTP_INTERCEPTORS} from '@angular/common/http';
import {AuthInterceptor} from './auth.interceptor';
import {AuthGuard, NoAuthGuard} from './guards/auth.guard';
import {AuthService} from './services/auth.service';
import {InboxService} from './services/inbox.service';
import {RepoService} from './services/repo.service';
import {PreviewComponent} from './components/preview/preview.component';
import { DocumentComponent } from './components/document/document.component';
import {PdfJsViewerModule} from 'ng2-pdfjs-viewer';
import {ClarityModule} from '@clr/angular';


@NgModule({
  declarations: [
    PreviewComponent,
    DocumentComponent
  ],
  imports: [
    CommonModule,
    ClarityModule,
    PdfJsViewerModule,
  ],
  exports: [
    PreviewComponent,
    DocumentComponent,
  ],
  providers: [
    AuthService,
    InboxService,
    RepoService,
    AuthGuard,
    NoAuthGuard,
    {provide: HTTP_INTERCEPTORS, useClass: AuthInterceptor, multi: true},
  ]
})
export class SharedModule {
}
