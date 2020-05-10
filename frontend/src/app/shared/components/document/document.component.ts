import {Component, Input, OnInit, ViewChild} from '@angular/core';
import {RepoService} from '../../services/repo.service';
import {PdfJsViewerComponent} from 'ng2-pdfjs-viewer';

@Component({
  selector: 'app-document',
  templateUrl: './document.component.html',
  styleUrls: ['./document.component.less']
})
export class DocumentComponent implements OnInit {

  @Input()
  public id: string;

  @ViewChild('viewer')
  public viewer: PdfJsViewerComponent;

  constructor(private repo: RepoService) {
  }

  ngOnInit(): void {
    this.repo.document(this.id)
      .subscribe(data => {
        this.viewer.pdfSrc = data;
        this.viewer.downloadFileName = this.id;
        this.viewer.refresh();
      });
  }
}
