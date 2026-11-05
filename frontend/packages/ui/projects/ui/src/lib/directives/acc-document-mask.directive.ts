import { Directive, HostListener, ElementRef, inject } from '@angular/core';
import { formatDocument, sanitizeDocument } from '@accurit/shared';

@Directive({
  selector: 'input[accDocMask]',
  standalone: true,
})
export class AccDocumentMaskDirective {
  private readonly el = inject(ElementRef<HTMLInputElement>);

  @HostListener('input', ['$event'])
  onInput(event: Event): void {
    const input = this.el.nativeElement;
    const raw = sanitizeDocument(input.value).slice(0, 14);
    const formatted = formatDocument(raw);

    if (input.value !== formatted) {
      input.value = formatted;
      input.dispatchEvent(new Event('input'));
    }
  }

  @HostListener('blur')
  onBlur(): void {
    const input = this.el.nativeElement;
    input.value = formatDocument(input.value);
  }
}

