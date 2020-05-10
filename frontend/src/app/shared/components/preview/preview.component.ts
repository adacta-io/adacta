import {Component, Input, OnChanges, OnInit} from '@angular/core';
import {Observable} from 'rxjs';
import {RepoService} from '../../services/repo.service';

@Component({
  selector: 'app-preview',
  templateUrl: './preview.component.html',
  styleUrls: ['./preview.component.less']
})
export class PreviewComponent implements OnChanges {

  @Input()
  public id: string;

  public src: string;

  constructor(private repo: RepoService) {
  }

  ngOnChanges(): void {
    this.repo.preview(this.id).subscribe(data => this.src = data);
  }
}
