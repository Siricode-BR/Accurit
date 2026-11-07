import { ChangeDetectionStrategy, Component, input } from '@angular/core';

/** Título + subtítulo da página; o conteúdo projetado vira a área de ações (botões). */
@Component({
  selector: 'acc-page-header',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './acc-page-header.component.html',
  styleUrl: './acc-page-header.component.css',
})
export class AccPageHeaderComponent {
  readonly title = input.required<string>();
  readonly subtitle = input<string | null>(null);
}
